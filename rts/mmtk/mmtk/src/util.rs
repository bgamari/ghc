use crate::edges::GHCEdge;
use crate::edges::Slot;
use crate::types::*;
use mmtk::util::ObjectReference;

pub unsafe fn offset_bytes<T>(ptr: *mut T, n: isize) -> *mut T {
    ptr.cast::<u8>().offset(n).cast()
}

pub unsafe fn offset_words<T>(ptr: *mut T, n: isize) -> *mut T {
    ptr.cast::<StgWord>().offset(n).cast()
}

/// Compute a pointer to a structure from an offset relative
/// to the end of another structure.
pub unsafe fn offset_from_end<Src, Target>(ptr: &Src, offset: isize) -> *const Target {
    let end = (ptr as *const Src).offset(1);
    (end as *const u8).offset(offset).cast()
}

/// Helper function to assist debugging
#[inline(never)]
#[cfg(feature = "mmtk_ghc_debug")]
pub fn push_slot(ptr: Slot) {
    push_node(unsafe { (*ptr.0).to_object_reference() });
    ()
}

/// Helper function to assist debugging
#[inline(never)]
#[cfg(feature = "mmtk_ghc_debug")]
pub fn push_node(_ptr: mmtk::util::ObjectReference) {
    unsafe { crate::ghc::push_node(_ptr.to_raw_address().to_ptr()) };
}

/// Helper function to push (standard StgClosure) edge to root packet
pub fn push_root(roots: &mut Vec<GHCEdge>, slot: Slot) {
    #[cfg(feature = "mmtk_ghc_debug")]
    push_slot(slot);

    roots.push(GHCEdge::from_closure_ref(slot))
}

pub fn assert_reachable(obj: ObjectReference) {
    if obj.is_in_any_space() {
        assert!(obj.is_reachable(), "{:?} not reachable", obj);
    }
}
