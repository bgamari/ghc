use crate::stg_closures::TaggedClosureRef;
use crate::stg_info_table::*;
use mmtk::util::constants::LOG_BYTES_IN_ADDRESS;
use mmtk::util::{Address, ObjectReference};
use mmtk::vm::edge_shape::{Edge, MemorySlice};

/// A pointer to a pointer to a heap object
/// i.e. a pointer to a field of a heap object
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Slot(
    pub *mut TaggedClosureRef
);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GHCEdge {
    /// An edge corresponding to a pointer field of a closure
    ClosureRef(Slot), // TODO: Use Atomic<...>

    /// An edge corresponding to the SRT of an info table.
    /// Precondition: The info table must have an SRT
    ThunkSrtRef(*mut StgThunkInfoTable),

    /// An edge corresponding to the SRT of an info table.
    /// Precondition: The info table must have an SRT
    RetSrtRef(*mut StgRetInfoTable),

    FunSrtRef(*mut StgFunInfoTable),
}

unsafe impl Send for GHCEdge {}

impl GHCEdge {
    // temporary to assist debugging
    #[inline(never)]
    pub fn from_closure_ref(cref: Slot) -> Self {
        let r = GHCEdge::ClosureRef(cref);
        assert!(r.load().to_raw_address().as_usize() & 0xffff == 0x0cb0);
        // assert!(false);
        r
    }
}

impl Edge for GHCEdge {
    fn load(&self) -> ObjectReference {
        match self {
            GHCEdge::ClosureRef(c) => unsafe {
                let cref: *mut TaggedClosureRef = c.0;
                let closure_ref: TaggedClosureRef = *cref;         // loads the pointer from the reference field
                let addr: Address = closure_ref.to_address();   // untags the pointer
                ObjectReference::from_raw_address(addr)         // converts it to an mmtk ObjectReference
            },
            GHCEdge::ThunkSrtRef(info_tbl) => unsafe {
                let some_table = <*mut StgThunkInfoTable>::as_ref(*info_tbl);
                if let Some(table) = some_table {
                    match table.get_srt() {
                        Some(srt) => ObjectReference::from_raw_address(Address::from_ptr(srt)),
                        None => panic!("Pushed SrtRef edge for info table without SRT"),
                    }
                } else {
                    panic!("Pushed SrtRef edge without info table")
                }
            }
            GHCEdge::RetSrtRef(info_tbl) => unsafe {
                let some_table = <*mut StgRetInfoTable>::as_ref(*info_tbl);
                if let Some(table) = some_table {
                    match table.get_srt() {
                        Some(srt) => ObjectReference::from_raw_address(Address::from_ptr(srt)),
                        None => panic!("Pushed SrtRef edge for info table without SRT"),
                    }
                } else {
                    panic!("PUshed SrtRef edge without info table")
                }
            }
            GHCEdge::FunSrtRef(info_tbl) => unsafe {
                let some_table = <*mut StgFunInfoTable>::as_ref(*info_tbl);
                if let Some(table) = some_table {
                    match table.get_srt() {
                        Some(srt) => ObjectReference::from_raw_address(Address::from_ptr(srt)),
                        None => panic!("Pushed SrtRef edge for info table without SRT"),
                    }
                } else {
                    panic!("PUshed SrtRef edge without info table")
                }
            }
        }
    }

    fn store(&self, object: ObjectReference) {
        match self {
            GHCEdge::ClosureRef(c) => unsafe {
                *(c.0) = TaggedClosureRef::from_address(object.to_raw_address());
            },
            GHCEdge::FunSrtRef(_) | GHCEdge::ThunkSrtRef(_) | GHCEdge::RetSrtRef(_) => {
                panic!("Attempted to store into an SrtRef");
            }
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GHCVMMemorySlice(*mut [ObjectReference]);

unsafe impl Send for GHCVMMemorySlice {}

impl MemorySlice for GHCVMMemorySlice {
    type Edge = GHCEdge;
    type EdgeIterator = GHCVMMemorySliceIterator;

    fn iter_edges(&self) -> Self::EdgeIterator {
        GHCVMMemorySliceIterator {
            cursor: unsafe { (*self.0).as_mut_ptr_range().start },
            limit: unsafe { (*self.0).as_mut_ptr_range().end },
        }
    }

    fn start(&self) -> Address {
        Address::from_ptr(unsafe { (*self.0).as_ptr_range().start })
    }

    fn bytes(&self) -> usize {
        unsafe { (*self.0).len() * std::mem::size_of::<ObjectReference>() }
    }

    fn copy(src: &Self, tgt: &Self) {
        debug_assert_eq!(src.bytes(), tgt.bytes());
        debug_assert_eq!(
            src.bytes() & ((1 << LOG_BYTES_IN_ADDRESS) - 1),
            0,
            "bytes are not a multiple of words"
        );
        // Raw memory copy
        unsafe {
            let words = tgt.bytes() >> LOG_BYTES_IN_ADDRESS;
            let src = src.start().to_ptr::<usize>();
            let tgt = tgt.start().to_mut_ptr::<usize>();
            std::ptr::copy(src, tgt, words)
        }
    }
}

pub struct GHCVMMemorySliceIterator {
    cursor: *mut ObjectReference,
    limit: *mut ObjectReference,
}

impl Iterator for GHCVMMemorySliceIterator {
    type Item = GHCEdge;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.limit {
            None
        } else {
            // TODO: next object
            // let edge = self.cursor;
            // self.cursor = unsafe { self.cursor.add(1) };
            // Some(GHCEdge::Simple(SimpleEdge::from_address(
            //     Address::from_ptr(edge),
            // )))
            None
        }
    }
}