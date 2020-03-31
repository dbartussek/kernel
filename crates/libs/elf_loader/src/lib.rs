#![no_std]

pub(crate) mod analysis;
pub mod loaded_object;
pub mod parameters;
pub(crate) mod relocations;

use crate::{
    analysis::elf_address_range, loaded_object::LoadedObject,
    parameters::LoadParameters, relocations::apply_relocations,
};
use core::{ops::Range, slice::from_raw_parts_mut};
use goblin::elf::{program_header::PT_LOAD, Elf};
use x86_64::VirtAddr;

fn range_size(r: &Range<usize>) -> usize {
    r.end - r.start
}

fn load_sections(
    elf: &Elf,
    binary: &[u8],
    buffer: &mut [u8],
    elf_base: VirtAddr,
) {
    for header in elf
        .program_headers
        .iter()
        .filter(|header| header.p_type == PT_LOAD)
    {
        let memory_range: Range<usize> = header.vm_range();
        let file_range: Range<usize> = header.file_range();

        // At what offset into our buffer is this section?
        let memory_base = memory_range.start - (elf_base.as_u64() as usize);

        // How big is this section in memory?
        let memory_size = range_size(&memory_range);

        // How big is this section in the binary?
        let file_size = range_size(&file_range);

        assert!(file_size <= memory_size);

        (&mut buffer[memory_base..(memory_base + file_size)])
            .copy_from_slice(&binary[file_range]);
    }
}

pub fn load<P>(binary: &[u8], mut parameters: P) -> LoadedObject
where
    P: LoadParameters,
{
    match Elf::parse(binary) {
        Ok(elf) => {
            if !elf.is_64 {
                panic!("Elf is 32 bit");
            }
            if !elf.is_lib {
                panic!("Elf is not relocatable");
            }

            let elf_address_range = elf_address_range(&elf);
            let (memory, relocation_location) = parameters
                .allocate_pages(
                    (elf_address_range.end - elf_address_range.start) as usize,
                )
                .unwrap();

            let mut buffer = unsafe {
                from_raw_parts_mut(
                    memory.start.start_address().as_mut_ptr::<u8>(),
                    (memory.end.start_address() - memory.start.start_address())
                        as usize,
                )
            };

            for it in buffer.iter_mut() {
                *it = 0;
            }

            load_sections(&elf, binary, &mut buffer, elf_address_range.start);

            let load_base = relocation_location.start.start_address();

            apply_relocations(
                &elf,
                &mut buffer,
                elf_address_range.start,
                load_base,
            );

            let entry = {
                let entry = VirtAddr::new(elf.entry) - elf_address_range.start;
                assert!(
                    entry < elf_address_range.end - elf_address_range.start
                );
                load_base + entry
            };

            LoadedObject {
                memory,
                relocation_location,
                entry,
            }
        },
        Err(e) => unimplemented!("Elf parse error: {:?}", e),
    }
}
