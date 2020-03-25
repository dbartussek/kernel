pub mod allocator;

pub use self::allocator::*;
use core::{alloc::Layout, ptr::NonNull};

pub trait OwnerCheck {
    fn is_owner(&self, ptr: NonNull<u8>, layout: Layout) -> bool;
}
