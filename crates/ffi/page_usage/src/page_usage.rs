#[repr(transparent)]
#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Debug, Hash)]
pub struct PageUsageRawType(u64);

impl PageUsageRawType {
    pub const fn from_category_and_data(category: u32, data: u32) -> Self {
        PageUsageRawType(((category as u64) << 32) | (data as u64))
    }

    pub const fn from_category(category: u32) -> Self {
        PageUsageRawType::from_category_and_data(category, 0)
    }

    pub const fn category(self) -> u32 {
        (self.0 >> 32) as u32
    }

    pub const fn data(self) -> u32 {
        self.0 as u32
    }

    pub const fn to_category_and_data(self) -> (u32, u32) {
        (self.category(), self.data())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PageUsage {
    Empty,
    Unusable,

    /// Reference count is number of references - 1
    PageTableRoot {
        reference_count: u32,
    },

    /// Reference count is number of references - 1
    PageTable {
        reference_count: u32,
    },

    KernelStack {
        thread: u32,
    },

    Custom(u32),
}

impl PageUsage {
    const TAG_CUSTOM: u32 = core::u32::MAX;
    const TAG_EMPTY: u32 = 0;
    const TAG_KERNEL_STACK: u32 = 4;
    const TAG_PAGE_TABLE: u32 = 3;
    const TAG_PAGE_TABLE_ROOT: u32 = 2;
    const TAG_UNUSABLE: u32 = 1;

    pub fn to_raw(self) -> Option<PageUsageRawType> {
        Some(match self {
            PageUsage::Empty => {
                PageUsageRawType::from_category(Self::TAG_EMPTY)
            },
            PageUsage::Unusable => {
                PageUsageRawType::from_category(Self::TAG_UNUSABLE)
            },

            PageUsage::PageTableRoot { reference_count } => {
                PageUsageRawType::from_category_and_data(
                    Self::TAG_PAGE_TABLE_ROOT,
                    reference_count,
                )
            },
            PageUsage::PageTable { reference_count } => {
                PageUsageRawType::from_category_and_data(
                    Self::TAG_PAGE_TABLE,
                    reference_count,
                )
            },

            PageUsage::KernelStack { thread } => {
                PageUsageRawType::from_category_and_data(
                    Self::TAG_KERNEL_STACK,
                    thread,
                )
            },

            PageUsage::Custom(i) => {
                PageUsageRawType::from_category_and_data(Self::TAG_CUSTOM, i)
            },
        })
    }

    pub fn from_raw(value: PageUsageRawType) -> Option<Self> {
        Some(match value.category() {
            Self::TAG_EMPTY => PageUsage::Empty,
            Self::TAG_UNUSABLE => PageUsage::Unusable,

            Self::TAG_PAGE_TABLE_ROOT => PageUsage::PageTableRoot {
                reference_count: value.data(),
            },
            Self::TAG_PAGE_TABLE => PageUsage::PageTable {
                reference_count: value.data(),
            },

            Self::TAG_KERNEL_STACK => PageUsage::KernelStack {
                thread: value.data(),
            },

            Self::TAG_CUSTOM => PageUsage::Custom(value.data()),

            _ => return None,
        })
    }

    pub fn is_empty(self) -> bool {
        self == PageUsage::Empty
    }
}
