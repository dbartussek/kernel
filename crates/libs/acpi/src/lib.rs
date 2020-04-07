#![no_std]

use uefi::Guid;
use x86_64::PhysAddr;

pub const RSDP2_GUID: Guid = Guid::from_values(
    0x8868E871,
    0xE4F1,
    0x11D3,
    0xBC22,
    [0x00, 0x80, 0xC7, 0x3C, 0x88, 0x81],
);

#[repr(C)]
pub struct RootSystemDescriptionPointer {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub root_system_descriptor_table_address: u32,
}

#[repr(C)]
pub struct RootSystemDescriptionPointer2 {
    pub first: RootSystemDescriptionPointer,

    pub length: u32,
    pub extended_system_descriptor_table_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

impl RootSystemDescriptionPointer {
    fn check_signature(&self) -> bool {
        &self.signature == "RSD PTR ".as_bytes()
    }

    pub fn check_valid(&self) -> bool {
        self.check_signature()
    }
}

impl RootSystemDescriptionPointer2 {
    pub fn check_valid(&self) -> bool {
        self.first.check_valid()
    }

    pub fn extended_system_descriptor_table_address(&self) -> PhysAddr {
        PhysAddr::new(self.extended_system_descriptor_table_address)
    }
}
