use crate::traits::Allocator;
use core::{
    alloc::{AllocErr, GlobalAlloc, Layout},
    ptr::NonNull,
};

pub struct SizeDeciding<S, L, const THRESHOLD: usize> {
    small: S,
    large: L,
}

impl<S, L, const THRESHOLD: usize> SizeDeciding<S, L, { THRESHOLD }> {
    pub const fn new(small: S, large: L) -> Self {
        SizeDeciding { small, large }
    }
}

impl<S, L, const THRESHOLD: usize> Default for SizeDeciding<S, L, { THRESHOLD }>
where
    S: Default,
    L: Default,
{
    fn default() -> Self {
        SizeDeciding::new(Default::default(), Default::default())
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
        if layout.size() <= THRESHOLD {
            self.small.alloc(layout)
        } else {
            self.large.alloc(layout)
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() <= THRESHOLD {
            self.small.dealloc(ptr, layout)
        } else {
            self.large.dealloc(ptr, layout)
        }
    }
}

unsafe impl<S, L, const THRESHOLD: usize> GlobalAlloc
    for SizeDeciding<S, L, { THRESHOLD }>
where
    S: GlobalAlloc,
    L: GlobalAlloc,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() <= THRESHOLD {
            self.small.alloc(layout)
        } else {
            self.large.alloc(layout)
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() <= THRESHOLD {
            self.small.dealloc(ptr, layout)
        } else {
            self.large.dealloc(ptr, layout)
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if layout.size() <= THRESHOLD {
            self.small.alloc_zeroed(layout)
        } else {
            self.large.alloc_zeroed(layout)
        }
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        if layout.size() <= THRESHOLD {
            self.small.realloc(ptr, layout, new_size)
        } else {
            self.large.realloc(ptr, layout, new_size)
        }
    }
}
