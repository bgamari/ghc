use std::sync::{Mutex, MutexGuard};
use crate::stg_closures::StgWeak;
use crate::ghc::{runSomeFinalizers, scheduleFinalizers, Capability};
use crate::GHCVM;
use mmtk::scheduler::GCWorker;
use mmtk::util::{ObjectReference, Address};
use mmtk::vm::{ObjectTracer, ObjectTracerContext};

pub struct WeakProcessor {
    weak_state: Mutex<WeakState>,
}
  
impl WeakProcessor {
    pub fn new() -> Self {
        WeakProcessor {
            weak_state: Mutex::new(WeakState::new())
        }
    }

    pub fn add_weak(&self, weak: *mut StgWeak) {
        let mut weak_state = self.weak_state.lock().unwrap();
        weak_state.add_weak(weak);
    }

    pub fn process_weak_refs(
            &self,
            worker: &mut GCWorker<GHCVM>,
            tracer_context: impl ObjectTracerContext<GHCVM>
    ) -> bool {
        // If this blocks, it is a bug.
        let mut weak_state: MutexGuard<WeakState> = 
            self.weak_state
            .try_lock()
            .expect("It's GC time. No mutators should hold this lock at this time.");

        weak_state.process_weak_refs(worker, tracer_context)
    }

    pub fn get_dead_weaks(&self) -> *const StgWeak {
        let mut weak_state = self.weak_state.lock().unwrap();
        weak_state.get_dead_weaks()
    }
}

struct WeakState {
    /// Weak references whose keys may be live.
    weak_refs: Vec<*mut StgWeak>,
    /// Weak references which we know have live keys
    live_weak_refs: Vec<*mut StgWeak>,
    /// Weak references which we need to finalize
    dead_weak_refs: Vec<*mut StgWeak>,
}

impl WeakState {
    pub fn new() -> Self {
        WeakState {
        weak_refs: vec!(),
        live_weak_refs: vec!(),
        dead_weak_refs: vec!(),
        }
    }

    pub fn add_weak(&mut self, weak: *mut StgWeak) {
        self.weak_refs.push(weak);
    }

    // Link together weak references with dead keys into a list and pass to RTS for finalization
    pub fn get_dead_weaks(&mut self) -> *const StgWeak {
        let mut last: *mut StgWeak = std::ptr::null_mut();
        for weak_ref in self.dead_weak_refs.iter() {
            let weak: &mut StgWeak = unsafe { &mut **weak_ref };
            weak.link = last;
            last = weak;
        }
        last
    }

    pub fn process_weak_refs(
            &mut self,
            worker: &mut GCWorker<GHCVM>,
            tracer_context: impl ObjectTracerContext<GHCVM>
    ) -> bool {
        // Weak references whose keys may still be live
        let mut remaining_weak_refs: Vec<*mut StgWeak> = vec!();
        let mut done = true;
        
        tracer_context.with_tracer(worker, |tracer| {
            for weak_ref in self.weak_refs.iter() {
                let weak: &mut StgWeak = unsafe { &mut **weak_ref };
                let obj_ref: ObjectReference = ObjectReference::from_raw_address(Address::from_ref(weak));
                if obj_ref.is_reachable() {
                    self.live_weak_refs.push(weak);
                    // TODO: For non-moving plan we might need to trace edge
                    tracer.trace_object(weak.key.to_object_reference());
                    tracer.trace_object(weak.finalizer.to_object_reference());
                    tracer.trace_object(weak.cfinalizers.to_object_reference());
                done = false;
                } else {
                    remaining_weak_refs.push(weak);
                }
            }
        });
        
        self.weak_refs = remaining_weak_refs;
        !done
    }

    /// Finalize dead weak references
    pub fn finish_gc_cycle(&mut self) {
        // Any weak references that remain on self.weak_refs at this point have unreachable keys.
        self.dead_weak_refs.append(&mut self.weak_refs);
        
        // Link together weak references with dead keys into a list and pass to RTS for finalization
        let mut last: *mut StgWeak = std::ptr::null_mut();
        for weak_ref in self.dead_weak_refs.iter() {
            let weak: &mut StgWeak = unsafe { &mut **weak_ref };
            weak.link = last;
            last = weak;
        }
        if !last.is_null() {
            let cap = std::ptr::null_mut::<Capability>();
            unsafe { scheduleFinalizers(cap, last) };
        }
        
        // Run any pending C finalizers
        unsafe { runSomeFinalizers(true) };
        
        // Prepare for next GC cycle
        self.weak_refs.append(&mut self.live_weak_refs);
        self.live_weak_refs = vec!();
    }
}