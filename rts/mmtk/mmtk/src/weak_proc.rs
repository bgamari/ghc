use std::sync::{Mutex, MutexGuard};
use std::collections::hash_set::HashSet;
use crate::stg_closures::StgWeak;
use crate::GHCVM;
use crate::ghc::{runCFinalizers, iter_capabilities, stg_NO_FINALIZER_closure};
use crate::util::{assert_reachable, push_node};
use mmtk::scheduler::GCWorker;
use mmtk::util::ObjectReference;
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

    pub fn finish_gc_cycle(&self) {
        let mut weak_state = self.weak_state.lock().unwrap();
        weak_state.finish_gc_cycle();
    }
}

struct WeakState {
    /// Weak references whose keys may be live.
    weak_refs: HashSet<*mut StgWeak>,
    /// Weak references which we know have live keys
    live_weak_refs: HashSet<*mut StgWeak>,
    /// Weak references which we need to finalize
    dead_weak_refs: HashSet<*mut StgWeak>,
}

impl WeakState {
    pub fn new() -> Self {
        WeakState {
            weak_refs: HashSet::new(),
            live_weak_refs: HashSet::new(),
            dead_weak_refs: HashSet::new(),
        }
    }

    pub fn add_weak(&mut self, weak: *mut StgWeak) {
        self.weak_refs.insert(weak);
    }

    // Link together weak references with dead keys into a list and pass to RTS for finalization
    pub fn get_dead_weaks(&mut self) -> *const StgWeak {
        let mut last: *mut StgWeak = std::ptr::null_mut();
        for weak_ref in self.dead_weak_refs.iter() {
            let weak: &mut StgWeak = unsafe { &mut **weak_ref };
            weak.link = last;
            last = weak;
        }
        self.dead_weak_refs.clear();
        last
    }

    pub fn process_weak_refs(
            &mut self,
            worker: &mut GCWorker<GHCVM>,
            tracer_context: impl ObjectTracerContext<GHCVM>
    ) -> bool {
        // Weak references whose keys may still be live
        let mut remaining_weak_refs: HashSet<*mut StgWeak> = HashSet::new();
        let mut done = true;
        
        tracer_context.with_tracer(worker, |tracer| {
            for weak_ref in self.weak_refs.iter() {
                let weak: &mut StgWeak = unsafe { &mut **weak_ref };
                let key_ref: ObjectReference = weak.key.to_object_reference();
                if key_ref.is_reachable() {
                    self.live_weak_refs.insert(weak);
                    // TODO: For non-moving plan we might need to trace edge
                    tracer.trace_object(weak.value.to_object_reference());
                    tracer.trace_object(weak.finalizer.to_object_reference());
                    tracer.trace_object(weak.cfinalizers.to_object_reference());

                    push_node(weak.value.to_object_reference());
                    push_node(weak.finalizer.to_object_reference());
                    push_node(weak.cfinalizers.to_object_reference());

                    done = false;
                } else {
                    remaining_weak_refs.insert(weak);
                }
            }
        });
        
        self.weak_refs = remaining_weak_refs;

        if done {
            // follow MarkWeak.c:collectWeakPtrs()
            // need to mark value and finalizer that are reachable from dead weak refs
            // so we can run finalizer
            tracer_context.with_tracer(worker, |tracer| {
                for w in self.weak_refs.iter() {
                    let weak: &mut StgWeak = unsafe { &mut **w };
                    if weak.cfinalizers.to_ptr() != 
                        unsafe{&stg_NO_FINALIZER_closure}
                    {
                        tracer.trace_object(weak.value.to_object_reference());
                        push_node(weak.value.to_object_reference());
                    }
                    tracer.trace_object(weak.finalizer.to_object_reference());
                    push_node(weak.finalizer.to_object_reference());
                }
            });
        }

        !done
    }


    /// Finalize dead weak references
    #[inline(never)]
    pub fn finish_gc_cycle(&mut self) {
        println!("====Finish GC cycle -- MMTK finalzer====");

        // Any weak references that remain on self.weak_refs at this point have unreachable keys.
        std::mem::swap(&mut self.weak_refs, &mut self.dead_weak_refs);

        // Prepare for next GC cycle
        std::mem::swap(&mut self.live_weak_refs, &mut self.weak_refs);

        for mut cap in iter_capabilities() {
            cap.weak_ptr_list_tl = std::ptr::null_mut();
            cap.weak_ptr_list_hd = std::ptr::null_mut();
        }
        // should be empty at this point
        assert!(self.live_weak_refs.is_empty());
        self.live_weak_refs.clear();

        for w in self.dead_weak_refs.iter() {
            let weak: &mut StgWeak = unsafe { &mut **w };
            assert_reachable(weak.value.to_object_reference());
            assert_reachable(weak.finalizer.to_object_reference());
            assert_reachable(weak.cfinalizers.to_object_reference());
            unsafe{runCFinalizers(weak.cfinalizers.to_ptr() as *const _)};
        }

        for w in self.weak_refs.iter() {
            let weak: &mut StgWeak = unsafe { &mut **w };
            assert_reachable(weak.value.to_object_reference());
            assert_reachable(weak.finalizer.to_object_reference());
            assert_reachable(weak.cfinalizers.to_object_reference());
        }
    }
}
