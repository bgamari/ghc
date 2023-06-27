use crate::ghc::*;
use crate::stg_closures::*;
use crate::stg_info_table::*;
use crate::types::StgClosureType::*;
use crate::GHCVM;
use crate::SINGLETON;
use mmtk::scheduler::GCWorker;
use mmtk::util::{opaque_pointer::*, Address, ObjectReference};
use mmtk::vm::ActivePlan;
use mmtk::Mutator;
use mmtk::ObjectQueue;
use mmtk::Plan;

use core::marker::PhantomData;

pub struct MutatorIterator<'a> {
    task: *const Task,

    /// true -> task.mmutator
    /// false -> task.rts_mutator
    which: bool,
    _marker: PhantomData<&'a Task>,
}

impl<'a> MutatorIterator<'a> {
    pub unsafe fn new() -> Self {
        MutatorIterator {
            task: all_tasks,
            which: true,
            _marker: PhantomData,
        }
    }
}

impl<'a> Iterator for MutatorIterator<'a> {
    type Item = &'a mut mmtk::Mutator<GHCVM>;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if !self.task.is_null() {
                let result = match self.which {
                    true => (*self.task).mmutator,
                    false => (*self.task).rts_mutator,
                };
                if !self.which {
                    self.task = (*self.task).all_next;
                    self.which = true;
                } else {
                    self.which = false;
                }
                Some(std::mem::transmute(result))
            } else {
                None
            }
        }
    }
}

pub struct VMActivePlan {}

impl ActivePlan<GHCVM> for VMActivePlan {
    fn global() -> &'static dyn Plan<VM = GHCVM> {
        SINGLETON.get_plan()
    }

    fn number_of_mutators() -> usize {
        // todo: number of tasks
        unsafe { n_capabilities as usize }
    }

    fn is_mutator(tls: VMThread) -> bool {
        unsafe { upcall_is_task(tls) }
    }

    fn mutator(tls: VMMutatorThread) -> &'static mut Mutator<GHCVM> {
        unsafe { &mut *upcall_get_mutator(tls) }
    }

    fn mutators<'a>() -> Box<dyn Iterator<Item = &'a mut Mutator<GHCVM>> + 'a> {
        unsafe {
            Box::new(MutatorIterator::new())
        }
    }

    fn vm_trace_object<Q: ObjectQueue>(
        queue: &mut Q,
        object: ObjectReference,
        _worker: &mut GCWorker<GHCVM>,
    ) -> ObjectReference {
        // Modelled after evacuate_static_object, returns true if this
        // is the first time the object has been visited in this GC.
        let mut evacuate_static = |static_link: &mut TaggedClosureRef| -> bool {
            let cur_static_flag = if crate::binding().get_static_flag() { 2 } else { 1 };
            let prev_static_flag = if crate::binding().get_static_flag() { 1 } else { 2 };
            let object_visited: bool = (static_link.get_tag() | prev_static_flag) == 3;
            if !object_visited {
                // N.B. We don't need to maintain a list of static objects, therefore ZERO
                *static_link =
                    TaggedClosureRef::from_address(Address::ZERO).set_tag(cur_static_flag);
                crate::util::push_node(object);
                enqueue_roots(queue, object);
            }
            !object_visited
        };

        // Modelled after scavenge_static() in Scav.c
        let tagged_ref = TaggedClosureRef::from_object_reference(object);
        let info_table = tagged_ref.get_info_table();

        use crate::stg_closures::Closure::*;
        match tagged_ref.to_closure() {
            ThunkStatic(thunk) => {
                // if srt != 0
                if let Some(_) = StgThunkInfoTable::from_info_table(info_table).get_srt() {
                    let static_link_ref = &mut thunk.static_link;
                    evacuate_static(static_link_ref);
                }
            }
            FunStatic(fun) => {
                let info_table = StgFunInfoTable::from_info_table(info_table);
                let srt = info_table.get_srt();
                let ptrs = unsafe { info_table.i.layout.payload.ptrs };
                // if srt != 0 || ptrs != 0
                if srt.is_some() || (ptrs != 0) {
                    let offset = unsafe { ptrs + info_table.i.layout.payload.nptrs };
                    let static_link_ref = fun.payload.get_ref(offset as usize);
                    evacuate_static(static_link_ref);
                }
            }
            IndirectStatic(ind) => {
                evacuate_static(&mut ind.static_link);
            }
            Constr(constr) => {
                if (info_table.type_ != CONSTR_0_1)
                    && (info_table.type_ != CONSTR_0_2)
                    && (info_table.type_ != CONSTR_NOCAF)
                {
                    let offset =
                        unsafe { info_table.layout.payload.ptrs + info_table.layout.payload.nptrs };
                    let static_link_ref = constr.payload.get_ref(offset as usize);
                    evacuate_static(static_link_ref);
                }
            }
            _ => panic!("invalid static closure"),
        };
        object
    }
}

pub fn enqueue_roots<Q: ObjectQueue>(queue: &mut Q, object: ObjectReference) {
    #[cfg(feature = "mmtk_ghc_debug")]
    crate::util::push_node(object);

    queue.enqueue(object);
}
