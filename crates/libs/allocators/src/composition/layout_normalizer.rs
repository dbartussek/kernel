use crate::traits::Allocator;
use core::{
    alloc::{AllocErr, AllocRef, CannotReallocInPlace, GlobalAlloc, Layout},
    ptr::NonNull,
};
use x86_64::structures::paging::{PageSize, Size4KiB};

#[derive(Default)]
pub struct LayoutNormalizer<A> {
    inner: A,
}

impl<A> LayoutNormalizer<A> {
    pub const fn new(inner: A) -> Self {
        LayoutNormalizer { inner }
    }

    pub fn into_inner(self) -> A {
        self.inner
    }

    fn normalize(layout: Layout) -> Layout {
        let page_size = Size4KiB::SIZE as usize;

        if layout.align() < page_size {
            // If the alignment is less than a page:
            // Just pad the allocation size to the alignment
            layout.pad_to_align()
        } else if layout.size() < page_size {
            // If the alignment is at least one page, but the allocation is for less:
            // Pad the allocation size to a page
            Layout::from_size_align(page_size, layout.align()).unwrap()
        } else {
            // If the allocation is at least a page or more:
            // Pass it through

            layout
        }
    }
}

impl<A> Allocator for LayoutNormalizer<A>
where
    A: Allocator,
{
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        self.inner.alloc(Self::normalize(layout))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.inner.dealloc(ptr, Self::normalize(layout))
    }
}

unsafe impl<A> AllocRef for LayoutNormalizer<A>
where
    A: AllocRef,
{
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        self.inner.alloc(Self::normalize(layout))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        self.inner.dealloc(ptr, Self::normalize(layout))
    }

    fn alloc_zeroed(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        self.inner.alloc_zeroed(Self::normalize(layout))
    }

    unsafe fn realloc(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        self.inner.realloc(ptr, Self::normalize(layout), new_size)
    }

    unsafe fn realloc_zeroed(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        self.inner
            .realloc_zeroed(ptr, Self::normalize(layout), new_size)
    }

    unsafe fn grow_in_place(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, CannotReallocInPlace> {
        self.inner
            .grow_in_place(ptr, Self::normalize(layout), new_size)
    }

    unsafe fn grow_in_place_zeroed(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, CannotReallocInPlace> {
        self.inner
            .grow_in_place_zeroed(ptr, Self::normalize(layout), new_size)
    }

    unsafe fn shrink_in_place(
        &mut self,
        ptr: NonNull<u8>,
        layout: Layout,
        new_size: usize,
    ) -> Result<usize, CannotReallocInPlace> {
        self.inner
            .shrink_in_place(ptr, Self::normalize(layout), new_size)
    }
}

unsafe impl<A> GlobalAlloc for LayoutNormalizer<A>
where
    A: GlobalAlloc,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.inner.alloc(Self::normalize(layout))
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, Self::normalize(layout))
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        self.inner.alloc_zeroed(Self::normalize(layout))
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        self.inner.realloc(ptr, Self::normalize(layout), new_size)
    }
}
