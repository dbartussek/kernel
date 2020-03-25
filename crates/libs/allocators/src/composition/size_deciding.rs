use crate::traits::Allocator;
use core::{
    alloc::{AllocErr, Layout},
    ptr::NonNull,
};

pub struct SizeDeciding<S, L, const THRESHOLD: usize>
where
    S: Allocator,
    L: Allocator,
{
    small: S,
    large: L,
}

impl<S, L, const THRESHOLD: usize> SizeDeciding<S, L, { THRESHOLD }>
where
    S: Allocator,
    L: Allocator,
{
    pub fn new(small: S, large: L) -> Self {
        SizeDeciding { small, large }
    }
}

impl<S, L, const THRESHOLD: usize> Allocator
    for SizeDeciding<S, L, { THRESHOLD }>
where
    S: Allocator,
    L: Allocator,
{
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        if layout.size() < THRESHOLD {
            self.small.alloc(layout)
        } else {
            self.large.alloc(layout)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() < THRESHOLD {
            self.small.dealloc(ptr, layout)
        } else {
            self.large.dealloc(ptr, layout)
        }
    }
}
