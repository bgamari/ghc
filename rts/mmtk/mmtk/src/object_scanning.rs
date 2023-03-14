use crate::edges::{GHCEdge, Slot};
use crate::ghc::*;
use crate::stg_closures::*;
use crate::stg_info_table::*;
use crate::types::*;
use crate::stg_closures::IsClosureRef;
use crate::util::{push_root, push_slot};
use crate::types::StgClosureType::{CONSTR_0_1, BLACKHOLE};
use mmtk::vm::EdgeVisitor;
use std::cmp::min;
use std::mem::size_of;

pub fn scan_closure_payload<EV: EdgeVisitor<GHCEdge>>(
    // _tls: VMWorkerThread,
    payload: &ClosurePayload,
    n_ptrs: u32,
    ev: &mut EV,
) {
    for n in 0..n_ptrs {
        let edge = payload.get_ref(n as usize);
        visit(ev, edge);
    }
}

/// Helper function to visit (standard StgClosure) edge
pub fn visit<EV: EdgeVisitor<GHCEdge>, Ref: IsClosureRef>(ev: &mut EV, slot: &mut Ref) {
    let s: Slot = IsClosureRef::to_tagged_closure_ref(slot);

    #[cfg(feature = "indirection_shortcutting")]
    let closure_ref = indirection_shortcutting(s);
    #[cfg(not(feature = "indirection_shortcutting"))]
    let closure_ref = s.get();

    let itbl = closure_ref.get_info_table();
    if itbl.type_ == CONSTR_0_1 {
        #[cfg(feature = "small_obj_optimisation")]
        if small_obj_optimisation(closure_ref) { return }
    }

    #[cfg(feature = "mmtk_ghc_debug")]
    push_slot(s);

    ev.visit_edge(GHCEdge::from_closure_ref(s));
}

/// Given a slot, we return the final indirectee of the chain
/// We update the slot to point to the final indirectee (Evac.c:938)
#[cfg(feature = "indirection_shortcutting")]
fn indirection_shortcutting(mut slot: Slot) -> TaggedClosureRef {
    let mut closure_ref: TaggedClosureRef = slot.get();

    loop {
        let itbl = closure_ref.get_info_table();

        // check if p is a blackhole, if so, we can try update it to its indirectee
        if itbl.type_ == BLACKHOLE {
            let indir: TaggedClosureRef = unsafe { (*(closure_ref.to_ptr() as *const StgInd)).indirectee };
            // if tag is zero, then we have to further check whether if indir is a blackhole
            let is_indirection: bool = if indir.get_tag() != 0 { true } else {
                let r_itbl = indir.get_info_table() as *const StgInfoTable;
                // if infotable of indir is one of these types, it means that indir is a blackhole,
                // so p does not have a updated value yet, end of the chain
                let is_blackhole = r_itbl == unsafe {stg_TSO_info} ||
                                r_itbl == unsafe {stg_WHITEHOLE_info} || 
                                r_itbl == unsafe {stg_BLOCKING_QUEUE_CLEAN_info} ||
                                r_itbl == unsafe {stg_BLOCKING_QUEUE_DIRTY_info};
                is_blackhole
            };
            if is_indirection {
                slot.set(indir);
                closure_ref = indir;
                continue;
            }
        }
        break;
    }
    closure_ref
}

/// Return true if optimisation is applied
/// Otherwise return false, in this case we need to enqueue the edge
#[cfg(feature = "small_obj_optimisation")]
fn small_obj_optimisation(closure_ref: TaggedClosureRef) -> bool {
    if let Some(c) = is_intlike_closure(closure_ref) {
        *closure_ref.get_payload_ref(0) = c;
        return true;
    } 
    else if let Some(c) = is_charlike_closure(closure_ref) {
        *closure_ref.get_payload_ref(0) = c;
        return true;
    }
    false
}

#[allow(non_snake_case)]
pub fn scan_TSO<EV: EdgeVisitor<GHCEdge>>(
    // _tls: VMWorkerThread,
    tso: &mut StgTSO,
    ev: &mut EV,
) {
    // update the pointer from the InCall
    if !tso.bound.is_null() {
        visit(ev, unsafe { &mut (*(*tso).bound).tso });
    }

    visit(ev, &mut tso.blocked_exceptions);
    visit(ev, &mut tso.blocking_queue);
    visit(ev, &mut tso.trec);
    visit(ev, &mut tso.stackobj);
    visit(ev, &mut tso.link);

    if tso.why_blocked == StgTSOBlocked::BLOCKED_ON_MVAR
        || tso.why_blocked == StgTSOBlocked::BLOCKED_ON_MVAR_READ
        || tso.why_blocked == StgTSOBlocked::BLOCKED_ON_BLACK_HOLE
        || tso.why_blocked == StgTSOBlocked::BLOCKED_ON_MSG_THROW_TO
        || tso.why_blocked == StgTSOBlocked::NOT_BLOCKED
    {
        unsafe {
            let edge = &mut tso.block_info.closure;
            visit(ev, edge);
        }
    }

    // TODO: GC should not trace (related to weak pointer)
    visit(ev, &mut tso.global_link);

    visit(ev, &mut tso.tso_link_prev);
    visit(ev, &mut tso.tso_link_next);
}

#[allow(non_snake_case)]
pub fn scan_PAP_payload<EV: EdgeVisitor<GHCEdge>>(
    // _tls: VMWorkerThread,
    fun_info: &StgFunInfoTable,
    payload: &ClosurePayload,
    size: usize,
    ev: &mut EV,
) {
    use StgFunType::*;
    debug_assert_ne!(fun_info.i.type_, StgClosureType::PAP);

    match fun_info.f.fun_type {
        ARG_GEN => unsafe {
            let small_bitmap: StgSmallBitmap = fun_info.f.bitmap.small_bitmap;
            scan_small_bitmap(payload, small_bitmap, size, ev);
        },
        ARG_GEN_BIG => unsafe {
            let large_bitmap: &StgLargeBitmap =
                &*(fun_info.f.bitmap.large_bitmap_ref.deref(fun_info));
            scan_large_bitmap(payload, large_bitmap, size, ev);
        },
        // TODO: handle ARG_BCO case
        _ => {
            let small_bitmap = StgFunType::get_small_bitmap(&fun_info.f.fun_type);
            scan_small_bitmap(payload, small_bitmap, size, ev);
        }
    }
}

static MUT_ARR_PTRS_CARD_BITS: usize = 7;

/// Scan mutable arrays of pointers
/// See rts/sm/Scav.c:scavenge_mut_arr_ptrs()
pub unsafe fn scan_mut_arr_ptrs<EV: EdgeVisitor<GHCEdge>>(
    // _tls: VMWorkerThread,
    array: &StgMutArrPtrs,
    ev: &mut EV,
) {
    // number of cards in the array
    let n_cards: StgWord =
        (array.n_ptrs + (1 << MUT_ARR_PTRS_CARD_BITS) - 1) >> MUT_ARR_PTRS_CARD_BITS;

    // scan card 0..n-1
    for m in 0..n_cards - 1 {
        // m-th card, iterate through 2^MUT_ARR_PTRS_CARD_BITS many elements
        for p in m * (1 << MUT_ARR_PTRS_CARD_BITS)..(m + 1) * (1 << MUT_ARR_PTRS_CARD_BITS) {
            let edge = array.payload.get_ref(p);
            visit(ev, edge);

            // mark m-th card to 0
            let m_card_address: *const StgWord8 =
                (array.payload.get(array.n_ptrs).to_ptr() as usize + m) as *const StgWord8;
            let mut _m_card_mark = &*m_card_address;
            _m_card_mark = &0;
        }
    }

    // scan the last card (no need to scan entirely)
    for p in (n_cards - 1) * (1 << MUT_ARR_PTRS_CARD_BITS)..array.n_ptrs {
        let edge = array.payload.get_ref(p);
        visit(ev, edge);

        // mark m-th card to 0
        let m_card_address: *const StgWord8 =
            (array.payload.get(array.n_ptrs).to_ptr() as usize + (n_cards - 1)) as *const StgWord8;
        let mut _m_card_mark = &*m_card_address;
        _m_card_mark = &0;
    }

    // TODO: use the optimised version later for card marking
    // (bool: whether there's an inter generation pointer (old to young))
}

/// See rts/sm/Scav.c:scavenge_small_bitmap()
pub fn scan_small_bitmap<EV: EdgeVisitor<GHCEdge>>(
    // _tls: VMWorkerThread,
    payload: &ClosurePayload,
    small_bitmap: StgSmallBitmap,
    size: usize,
    ev: &mut EV,
) {
    let mut bitmap = small_bitmap.bits();

    for i in 0..size {
        if (bitmap & 1) == 0 {
            visit(ev, payload.get_ref(i));
        }
        bitmap = bitmap >> 1;
    }
}

/// See rts/sm/Scav.c:scavenge_large_bitmap()
pub fn scan_large_bitmap<EV: EdgeVisitor<GHCEdge>>(
    // _tls: VMWorkerThread,
    payload: &ClosurePayload,
    large_bitmap: &StgLargeBitmap,
    size: usize,
    ev: &mut EV,
) {
    // Bitmap may have more bits than `size` when scavenging PAP payloads
    // PAP n_args < fun.bitmap.size
    // AP n_args = fun.bitmap.size
    debug_assert!(size <= large_bitmap.size);

    let mut b: usize = 0;
    let mut i: usize = 0;
    while i < size {
        let mut bitmap = unsafe { *(large_bitmap.bitmap).get_w(b) };
        // word_len is the size is min(wordsize, (size_w - i) bits)
        let word_len = min(size - i, 8 * size_of::<StgWord>());
        i += word_len;
        for j in 0..word_len {
            if (bitmap & 1) == 0 {
                let edge = payload.get_ref(j);
                visit(ev, edge);
            }
            bitmap = bitmap >> 1;
        }
        b += 1;
    }
}

/// See rts/sm/Scav.c:scavenge_stack()
pub fn scan_stack<EV: EdgeVisitor<GHCEdge>>(stack: StackIterator, ev: &mut EV) {
    for stackframe in stack {
        use StackFrame::*;
        match stackframe {
            UPD_FRAME(frame) => {
                visit(ev, &mut frame.updatee);
            }
            RET_SMALL(frame, bitmap) => {
                let payload: &'static ClosurePayload = &(frame.payload);
                scan_small_bitmap(payload, bitmap, bitmap.size(), ev);
                let ret_itbl = unsafe { &mut *(frame.header.info_table.get_mut_ptr()) };
                scan_srt(ret_itbl, ev);
            }
            RET_BIG(frame, bitmap_ref) => {
                let payload: &'static ClosurePayload = &(frame.payload);
                let size: usize = bitmap_ref.size;
                scan_large_bitmap(payload, bitmap_ref, size, ev);
                let ret_itbl = unsafe { &mut *(frame.header.info_table.get_mut_ptr()) };
                scan_srt(ret_itbl, ev);
            }
            RET_FUN_SMALL(frame, bitmap) => {
                visit(ev, &mut frame.fun);
                let payload: &'static ClosurePayload = &(frame.payload);
                scan_small_bitmap(payload, bitmap, bitmap.size(), ev);
                let ret_itbl = unsafe { &mut *(frame.info_table.get_mut_ptr()) };
                scan_srt(ret_itbl, ev);
            }
            RET_FUN_LARGE(frame, bitmap_ref) => {
                visit(ev, &mut frame.fun);
                let payload: &'static ClosurePayload = &(frame.payload);
                let size: usize = bitmap_ref.size;
                scan_large_bitmap(payload, bitmap_ref, size, ev);
                let ret_itbl = unsafe { &mut *(frame.info_table.get_mut_ptr()) };
                scan_srt(ret_itbl, ev);
            }
            _ => panic!("Unexpected stackframe type {stackframe:?}"),
        }
    }
}

/// See (follow_srt) in rts/sm/Scav.c:scavenge_stack
pub fn scan_srt<EV: EdgeVisitor<GHCEdge>>(ret_info_table: &mut StgRetInfoTable, ev: &mut EV) {
    // TODO: only for major gc
    // TODO: non USE_INLINE_SRT_FIELD
    match ret_info_table.get_srt() {
        None => (),
        Some(_srt) => {

            #[cfg(feature = "mmtk_ghc_debug")]
            crate::util::push_node(
                mmtk::vm::edge_shape::Edge::load(
                    &GHCEdge::RetSrtRef(ret_info_table)));

            ev.visit_edge(GHCEdge::RetSrtRef(ret_info_table));
        }
    }
}

/// In the case of USE_INLINE_SRT_FIELD, SRT is reperesented using an offset,
/// so we cannot use the standard edge representation
pub fn scan_srt_thunk<EV: EdgeVisitor<GHCEdge>>(
    thunk_info_table: &mut StgThunkInfoTable,
    ev: &mut EV,
) {
    // TODO: only for major gc
    // TODO: non USE_INLINE_SRT_FIELD
    match thunk_info_table.get_srt() {
        None => (),
        Some(_srt) => {

            #[cfg(feature = "mmtk_ghc_debug")]
            crate::util::push_node(
                mmtk::vm::edge_shape::Edge::load(
                    &GHCEdge::ThunkSrtRef(thunk_info_table)));

            ev.visit_edge(GHCEdge::ThunkSrtRef(thunk_info_table));
        }
    }
}

pub fn scan_srt_fun<EV: EdgeVisitor<GHCEdge>>(fun_info_table: &mut StgFunInfoTable, ev: &mut EV) {
    // TODO: only for major gc
    // TODO: non USE_INLINE_SRT_FIELD
    match fun_info_table.get_srt() {
        None => (),
        Some(_srt) => {

            #[cfg(feature = "mmtk_ghc_debug")]
            crate::util::push_node(
                mmtk::vm::edge_shape::Edge::load(
                    &GHCEdge::FunSrtRef(fun_info_table)));

            ev.visit_edge(GHCEdge::FunSrtRef(fun_info_table));
        }
    }
}

/// Treat objects from SRT as roots
/// See rts/StablePtr.c/FOR_EACH_STABLE_PTR
pub fn get_stable_ptr_table_roots() -> Vec<GHCEdge> {
    unsafe {
        let mut roots: Vec<GHCEdge> = vec![];
        let tables: *mut spEntry = stable_ptr_table;
        let __end_ptr: *mut spEntry = tables.offset(SPT_size as isize);

        for table in iter_stable_ptr_table() {
            if (table.addr != 0 as *mut _)
                && ((table.addr < stable_ptr_table as *mut usize)
                    || (table.addr >= __end_ptr as *mut usize))
            {
                let edge_addr: *const *mut usize = &(table.addr) as *const *mut usize;
                let edge: Slot = Slot(edge_addr as *mut TaggedClosureRef);

                push_root(&mut roots, edge);
            }
        }
        roots
    }
}
