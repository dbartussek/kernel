use core::{
    alloc::{AllocErr, Layout},
    ptr::NonNull,
};

/// An owning allocator
///
/// Unlike AllocRef, this allocator may not be moved while there are allocations in it
pub trait Allocator {
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr>;

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout);
}
