use core::num::NonZeroU64;

pub type RawCoreId = NonZeroU64;
pub type OptionalRawCoreId = u64;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct CoreId {
    id: RawCoreId,
}

impl CoreId {
    pub fn from_full_id(id: RawCoreId) -> Self {
        CoreId { id }
    }

    pub fn from_optional_full_id(id: OptionalRawCoreId) -> Option<Self> {
        RawCoreId::new(id).map(Self::from_full_id)
    }

    pub fn full_id(&self) -> RawCoreId {
        self.id
    }

    pub fn optional_full_id(&self) -> OptionalRawCoreId {
        self.id.get()
    }

    pub fn optional_to_optional_full_id(
        this: Option<Self>,
    ) -> OptionalRawCoreId {
        this.map(|this| this.optional_full_id()).unwrap_or(0)
    }
}
