use std::sync::{Mutex, atomic::{AtomicBool, Ordering}};
use crate::weak_proc::WeakProcessor;

pub struct GHCBinding {
    /// Indicate the state of static object scanning cycle
    /// Check `evacuate_static_object()`
    pub static_flag: AtomicBool,
    pub weak_proc: Mutex<WeakProcessor>,
}

unsafe impl Sync for GHCBinding {}
unsafe impl Send for GHCBinding {}

impl GHCBinding {
    pub fn new() -> Self {
        GHCBinding {
            static_flag: false.into(),
            weak_proc: Mutex::new(WeakProcessor::new()),
        }
    }

    pub fn bump_static_flag(&self) {
        let current_value = self.static_flag.load(Ordering::Relaxed);
        self.static_flag.store(!current_value, Ordering::Relaxed);
    }

    pub fn get_static_flag(&self) -> bool {
        self.static_flag.load(Ordering::Relaxed)
    }
}
