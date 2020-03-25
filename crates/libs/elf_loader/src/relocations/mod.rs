use byteorder::{ByteOrder, LittleEndian};
use core::num::Wrapping;
use goblin::elf::{
    header::EM_X86_64,
    reloc::{Reloc, R_X86_64_RELATIVE},
    Elf,
};
use x86_64::VirtAddr;

#[derive(Copy, Clone, Debug)]
struct Relocation {
    data: Reloc,
}

impl Relocation {
    pub fn new(data: Reloc) -> Self {
        Relocation { data }
    }

    pub fn apply(
        &self,
        _elf_base: VirtAddr,
        load_base: VirtAddr,
        program: &mut [u8],
    ) {
        let offset = self.data.r_offset as usize;
        if offset >= program.len() {
            unimplemented!()
        }

        let position = &mut program[offset..];

        match self.data.r_type {
            R_X86_64_RELATIVE => {
                let value = VirtAddr::try_new(
                    (Wrapping(load_base.as_u64())
                        + Wrapping(self.data.r_addend.unwrap() as u64))
                    .0,
                )
                .unwrap();
                LittleEndian::write_u64(position, value.as_u64());
            },
            unknown => panic!(
                "Unknown relocation type: {}",
                goblin::elf::reloc::r_to_str(unknown, EM_X86_64),
            ),
        }
    }
}

fn relocations<'lt>(elf: &'lt Elf) -> impl 'lt + Iterator<Item = Relocation> {
    elf.dynrelas
        .iter()
        .chain(elf.dynrels.iter())
        .chain(elf.pltrelocs.iter())
        .map(Relocation::new)
}

pub fn apply_relocations(
    elf: &Elf,
    mut buffer: &mut [u8],
    elf_base: VirtAddr,
    load_base: VirtAddr,
) {
    for relocation in relocations(elf) {
        relocation.apply(elf_base, load_base, &mut buffer);
    }
}
