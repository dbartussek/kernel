use crate::traits::{Allocator, OwnerCheck};
use core::{
    alloc::{AllocErr, AllocRef, Layout},
    ptr::NonNull,
};
use spin::Mutex;

/// Create an AllocRef from a guarded Allocator reference
#[derive(Copy, Clone)]
pub struct MagicAllocRef<'a, A>
where
    A: Allocator,
{
    allocator: &'a Mutex<A>,
}

impl<'a, A> MagicAllocRef<'a, A>
where
    A: Allocator,
{
    pub fn new(allocator: &'a Mutex<A>) -> Self {
        MagicAllocRef { allocator }
    }

    pub fn allocator<F, R>(&self, function: F) -> R
    where
        F: FnOnce(&mut A) -> R,
    {
        let mut lock = self.allocator.lock();
        function(&mut lock)
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
