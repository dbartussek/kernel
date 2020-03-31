use crate::traits::{Allocator, OwnerCheck};
use core::{
    alloc::{AllocErr, Layout},
    ptr::NonNull,
};

pub struct FallBackAllocator<P, F>
where
    P: Allocator + OwnerCheck,
    F: Allocator,
{
    primary: P,
    fallback: F,
}

impl<P, F> FallBackAllocator<P, F>
where
    P: Allocator + OwnerCheck,
    F: Allocator,
{
    pub const fn new(primary: P, fallback: F) -> Self {
        FallBackAllocator { primary, fallback }
    }
}

impl<P, F> Allocator for FallBackAllocator<P, F>
where
    P: Allocator + OwnerCheck,
    F: Allocator,
{
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        match self.primary.alloc(layout) {
            Ok(r) => Ok(r),
            _ => self.fallback.alloc(layout),
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if self.primary.is_owner(ptr, layout) {
            self.primary.dealloc(ptr, layout)
        } else {
            self.fallback.dealloc(ptr, layout)
        }
    }
}

impl<P, F> OwnerCheck for FallBackAllocator<P, F>
where
    P: Allocator + OwnerCheck,
    F: Allocator + OwnerCheck,
{
    fn is_owner(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        self.primary.is_owner(ptr, layout)
            || self.fallback.is_owner(ptr, layout)
    }
}
