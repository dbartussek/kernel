#![no_std]

use ffi_utils::ffi_slice::FfiSliceMut;

pub type PageUsageRawType = i32;

#[repr(C)]
pub struct PhysicalMemoryMap<'buf> {
    buffer: FfiSliceMut<'buf, PageUsageRawType>,
    base: usize,
}

impl<'buf> PhysicalMemoryMap<'buf> {
    pub fn new(
        buffer: &'buf mut [PageUsageRawType],
        base: usize,
        value: PageUsage,
    ) -> Self {
        let value = value.to_raw().unwrap();

        for it in buffer.iter_mut() {
            *it = value;
        }

        PhysicalMemoryMap {
            buffer: buffer.into(),
            base,
        }
    }

    pub fn set(
        &mut self,
        page_index: usize,
        value: PageUsage,
    ) -> Option<PageUsage> {
        let value = value.to_raw()?;
        let base = self.base();

        self.buffer_mut()
            .get_mut(page_index - base)
            .map(|r| core::mem::replace(r, value))
            .map(|v| PageUsage::from_raw(v).unwrap())
    }

    pub fn get(&self, page_index: usize) -> Option<PageUsage> {
        self.buffer()
            .get(page_index)
            .map(|v| PageUsage::from_raw(*v).unwrap())
    }

    /// Consume self and return the underlying buffer
    ///
    /// This does not deallocate the buffer
    #[inline(always)]
    pub fn release_buffer(self) -> &'buf mut [PageUsageRawType] {
        self.buffer.into()
    }

    #[inline(always)]
    pub fn buffer(&self) -> &[PageUsageRawType] {
        self.buffer.as_slice()
    }

    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut [PageUsageRawType] {
        self.buffer.as_slice_mut()
    }

    #[inline(always)]
    pub fn pages(&self) -> usize {
        self.buffer.len()
    }

    #[inline(always)]
    pub fn base(&self) -> usize {
        self.base
    }

    pub fn iter<'lt>(&'lt self) -> impl 'lt + Iterator<Item = PageUsage> {
        self.buffer()
            .iter()
            .map(|v| PageUsage::from_raw(*v).unwrap())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PageUsage {
    Empty,
    Unusable,

    Custom(PageUsageRawType),
}

impl PageUsage {
    pub fn to_raw(self) -> Option<PageUsageRawType> {
        Some(match self {
            PageUsage::Empty => 0,
            PageUsage::Unusable => 1,
            PageUsage::Custom(i) if i < 0 => i,
            PageUsage::Custom(_) => return None,
        })
    }

    pub fn from_raw(value: PageUsageRawType) -> Option<Self> {
        Some(match value {
            0 => PageUsage::Empty,
            1 => PageUsage::Unusable,
            i if i < 0 => PageUsage::Custom(i),
            _ => return None,
        })
    }
}
