use crate::{
    traits::{Allocator, OwnerCheck},
    utils::bitset::BitSet,
};
use core::{
    alloc::{AllocErr, Layout},
    any::type_name,
    mem::MaybeUninit,
    ptr::NonNull,
};
use log::*;

const MAX_ALIGNMENT: usize = 16;

#[repr(align(16))]
struct Block<const BLOCK_SIZE: usize>([u8; BLOCK_SIZE]);

pub struct FixedBitMap<const BLOCK_SIZE: usize, const CAPACITY: usize> {
    bitmap: BitSet<{ CAPACITY }>,
    storage: [Block<{ BLOCK_SIZE }>; CAPACITY],
}

impl<const BLOCK_SIZE: usize, const CAPACITY: usize>
    FixedBitMap<{ BLOCK_SIZE }, { CAPACITY }>
{
    #[allow(clippy::uninit_assumed_init)]
    pub fn new() -> Self {
        FixedBitMap {
            bitmap: Default::default(),
            storage: unsafe { MaybeUninit::uninit().assume_init() },
        }
    }

    fn start_pointer(&self) -> *const u8 {
        self.storage.first().unwrap() as *const _ as *const u8
    }

    fn end_pointer(&self) -> *const u8 {
        self.storage.last().unwrap() as *const _ as *const u8
    }
}

impl<const BLOCK_SIZE: usize, const CAPACITY: usize> Default
    for FixedBitMap<{ BLOCK_SIZE }, { CAPACITY }>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const BLOCK_SIZE: usize, const CAPACITY: usize> Allocator
    for FixedBitMap<{ BLOCK_SIZE }, { CAPACITY }>
{
    fn alloc(
        &mut self,
        layout: Layout,
    ) -> Result<(NonNull<u8>, usize), AllocErr> {
        if layout.size() > BLOCK_SIZE || layout.align() > MAX_ALIGNMENT {
            return Err(AllocErr);
        }

        trace!("{}: allocating {:?}", type_name::<Self>(), layout);

        let allocation_index =
            self.bitmap.find_first_unset().ok_or(AllocErr)?;

        trace!("Index for allocation: {}", allocation_index);

        debug_assert!(allocation_index < CAPACITY);

        self.bitmap.insert(allocation_index);

        let ptr = NonNull::new(
            (&mut self.storage[allocation_index]) as *mut _ as *mut u8,
        )
        .unwrap();

        trace!("Allocated {:?}", ptr);

        Ok((ptr, BLOCK_SIZE))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        debug_assert!(layout.size() <= BLOCK_SIZE);
        debug_assert!(layout.align() <= MAX_ALIGNMENT);
        debug_assert!(self.is_owner(ptr, layout));

        let ptr = ptr.as_ptr() as *const Block<{ BLOCK_SIZE }>;
        let index = ptr.offset_from(self.storage.as_ptr()) as usize;

        debug_assert!(index < CAPACITY);
        debug_assert_eq!(self.bitmap.contains(index), Some(true));

        self.bitmap.remove(index);
    }
}

impl<const BLOCK_SIZE: usize, const CAPACITY: usize> OwnerCheck
    for FixedBitMap<{ BLOCK_SIZE }, { CAPACITY }>
{
    fn is_owner(&self, ptr: NonNull<u8>, layout: Layout) -> bool {
        if layout.size() > BLOCK_SIZE || layout.align() > MAX_ALIGNMENT {
            return false;
        }

        let ptr = ptr.as_ptr() as *const u8;

        ptr >= self.start_pointer() && ptr <= self.end_pointer()
    }
}
