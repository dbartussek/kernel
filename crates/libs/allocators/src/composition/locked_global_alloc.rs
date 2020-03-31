use crate::traits::Allocator;
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{null_mut, NonNull},
};
use kernel_spin::KernelMutex;

pub struct LockedGlobalAlloc<A> {
    inner: KernelMutex<A>,
}

impl<A> LockedGlobalAlloc<A> {
    pub const fn new(allocator: A) -> Self {
        LockedGlobalAlloc {
            inner: KernelMutex::new(allocator),
        }
    }

    pub fn into_inner(self) -> A {
        self.inner.into_inner()
    }
}

unsafe impl<A> GlobalAlloc for LockedGlobalAlloc<A>
where
    A: Allocator,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.lock(|allocator| {
            allocator
                .alloc(layout)
                .map(|(ptr, _)| ptr.as_ptr())
                .unwrap_or(null_mut())
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let Some(ptr) = NonNull::new(ptr) {
            self.inner.lock(|allocator| allocator.dealloc(ptr, layout))
        }
    }
}
