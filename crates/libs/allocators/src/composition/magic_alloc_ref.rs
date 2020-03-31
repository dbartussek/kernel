use crate::traits::{Allocator, OwnerCheck};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
    ptr::NonNull,
};
use kernel_spin::KernelMutex;

/// Create an AllocRef from a guarded Allocator reference
#[derive(Copy, Clone)]
pub struct MagicAllocRef<'a, A>
where
    A: Allocator,
{
    allocator: &'a KernelMutex<A>,
}

impl<'a, A> MagicAllocRef<'a, A>
where
    A: Allocator,
{
    pub const fn new(allocator: &'a KernelMutex<A>) -> Self {
        MagicAllocRef { allocator }
    }

    pub fn allocator<F, R>(&self, function: F) -> R
    where
        F: FnOnce(&mut A) -> R,
    {
        self.allocator.lock(function)
    }
}

unsafe impl<'a, A> AllocRef for MagicAllocRef<'a, A>
where
    A: Allocator,
{
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        self.allocator(|a| a.alloc(layout))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.allocator(|a| a.dealloc(ptr, layout))
    }
}

impl<'a, A> OwnerCheck for MagicAllocRef<'a, A>
where
    A: Allocator + OwnerCheck,
{
    fn is_owner(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        self.allocator(|a| a.is_owner(ptr, layout))
    }
}
