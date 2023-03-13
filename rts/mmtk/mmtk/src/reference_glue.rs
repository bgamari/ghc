use crate::GHCVM;
use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::ReferenceGlue;

pub struct VMReferenceGlue {}

impl ReferenceGlue<GHCVM> for VMReferenceGlue {
    type FinalizableType = ObjectReference;

    fn get_referent(_object: ObjectReference) -> ObjectReference {
        unimplemented!();
    }
    fn set_referent(_reff: ObjectReference, _referent: ObjectReference) {
        unimplemented!();
    }
    fn enqueue_references(_references: &[ObjectReference], _tls: VMWorkerThread) {
        unimplemented!();
    }
}
