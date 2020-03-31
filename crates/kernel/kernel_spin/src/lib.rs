#![no_std]

use core::{
    any::type_name,
    sync::atomic::{AtomicU64, Ordering},
};
use cpu_local_storage::{data::CoreId, get_core_id};
use x86_64::instructions::interrupts;

/// A Spinlock that disables interrupts while it is locked
///
/// Interrupts are disabled in critical sections to prevent deadlocks
#[derive(Default)]
pub struct KernelMutex<T> {
    mutex: spin::Mutex<T>,
    current_holder_id: AtomicU64,
}

impl<T> KernelMutex<T> {
    pub const fn new(data: T) -> Self {
        KernelMutex {
            mutex: spin::Mutex::new(data),
            current_holder_id: AtomicU64::new(0),
        }
    }

    /// Destroy the Mutex and return the current value
    pub fn into_inner(self) -> T {
        self.mutex.into_inner()
    }

    pub fn lock<F, R>(&self, function: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        interrupts::without_interrupts(move || {
            // If the core already holds the lock, we have a deadlock
            if self.holder() == Some(get_core_id()) {
                panic!(
                    "Recursive lock on type {} in core {:?}",
                    type_name::<T>(),
                    get_core_id()
                );
            }

            // Lock the Mutex
            let mut guard = self.mutex.lock();

            // Set the current lock holder so we can detect deadlocks
            self.set_holder(Some(get_core_id()));

            let result = function(&mut guard);

            // Reset the lock holder so we don't trigger "deadlocks"
            self.set_holder(None);

            result
        })
    }

    fn holder(&self) -> Option<CoreId> {
        let raw = self.current_holder_id.load(Ordering::Acquire);
        CoreId::from_optional_full_id(raw)
    }

    fn set_holder(&self, holder: Option<CoreId>) {
        let raw = CoreId::optional_to_optional_full_id(holder);
        self.current_holder_id.store(raw, Ordering::Release);
    }
}
