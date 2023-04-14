extern crate libc;
extern crate mmtk;
#[macro_use]
extern crate lazy_static;

use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use mmtk::vm::VMBinding;
use mmtk::MMTKBuilder;
use mmtk::MMTK;

use binding::GHCBinding;

pub mod active_plan;
pub mod api;
pub mod binding;
pub mod collection;
pub mod edges;
mod ghc;
pub mod object_model;
pub mod object_scanning;
pub mod reference_glue;
pub mod scanning;
pub mod stg_closures;
pub mod stg_info_table;
pub mod test;
pub mod types;
pub mod util;
pub mod weak_proc;     

#[cfg(test)]
mod tests;

#[derive(Default)]
pub struct GHCVM;

impl VMBinding for GHCVM {
    type VMObjectModel = object_model::VMObjectModel;
    type VMScanning = scanning::VMScanning;
    type VMCollection = collection::VMCollection;
    type VMActivePlan = active_plan::VMActivePlan;
    type VMReferenceGlue = reference_glue::VMReferenceGlue;
    type VMEdge = edges::GHCEdge;
    type VMMemorySlice = edges::GHCVMMemorySlice;

    /// Allowed maximum alignment in bytes.
    const MAX_ALIGNMENT: usize = 1 << 6;
}

/// This is used to ensure we initialize MMTk at a specified timingu
pub static MMTK_INITIALIZED: AtomicBool = AtomicBool::new(false);

lazy_static! {
    pub static ref BUILDER: Mutex<MMTKBuilder> = Mutex::new(MMTKBuilder::new());
    pub static ref SINGLETON: MMTK<GHCVM> = {
        let builder = BUILDER.lock().unwrap();
        debug_assert!(!MMTK_INITIALIZED.load(Ordering::SeqCst));
        let ret = mmtk::memory_manager::mmtk_init(&builder);
        MMTK_INITIALIZED.store(true, std::sync::atomic::Ordering::Relaxed);
        *ret
    };
}


/// Record global state for MMTk Binding
pub static BINDING: OnceCell<GHCBinding> = OnceCell::new();

pub fn binding<'b>() -> &'b GHCBinding {
    BINDING.get().expect("Attempt to use the binding before it is initialization")
}
