use core::mem::MaybeUninit;

type RawType = usize;
const RAW_TYPE_BITS: usize = core::mem::size_of::<RawType>() * 8;

#[derive(Copy, Clone)]
#[repr(transparent)]
struct BitData<const ARRAY_SIZE: usize>([RawType; ARRAY_SIZE]);

impl<const ARRAY_SIZE: usize> BitData<{ ARRAY_SIZE }> {
    pub fn new() -> Self {
        unsafe { BitData(MaybeUninit::zeroed().assume_init()) }
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct BitSet<const SIZE: usize>(
    BitData<{ (SIZE + RAW_TYPE_BITS - 1) / RAW_TYPE_BITS }>,
);

impl<const SIZE: usize> Default for BitSet<{ SIZE }> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> BitSet<{ SIZE }> {
    pub fn new() -> Self {
        BitSet(BitData::new())
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        SIZE
    }

    fn get_group_for(&self, index: usize) -> Option<RawType> {
        if index < self.len() {
            Some(unsafe { *(self.0).0.get_unchecked(index / RAW_TYPE_BITS) })
        } else {
            None
        }
    }

    fn get_group_for_mut(&mut self, index: usize) -> Option<&mut RawType> {
        if index < self.len() {
            Some(unsafe { (self.0).0.get_unchecked_mut(index / RAW_TYPE_BITS) })
        } else {
            None
        }
    }

    fn create_mask(index: usize) -> RawType {
        let index = (index % RAW_TYPE_BITS) as RawType;
        1 << index
    }

    pub fn contains(&self, index: usize) -> Option<bool> {
        Some((self.get_group_for(index)? & Self::create_mask(index)) != 0)
    }

    pub fn insert(&mut self, index: usize) -> Option<()> {
        *self.get_group_for_mut(index)? |= Self::create_mask(index);
        Some(())
    }

    pub fn remove(&mut self, index: usize) -> Option<()> {
        *self.get_group_for_mut(index)? &= !Self::create_mask(index);
        Some(())
    }

    pub fn set(&mut self, index: usize, value: bool) -> Option<()> {
        if value {
            self.insert(index)
        } else {
            self.remove(index)
        }
    }

    pub fn find_first_unset(&self) -> Option<usize> {
        for (group_index, group) in (self.0).0.iter().enumerate() {
            let trailing_ones = group.trailing_ones() as usize;

            if trailing_ones < RAW_TYPE_BITS {
                let index = group_index * RAW_TYPE_BITS + trailing_ones;
                return if index < SIZE { Some(index) } else { None };
            }
        }

        None
    }
}
