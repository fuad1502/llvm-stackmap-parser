use std::{fs, path::Path};

use goblin::elf::Elf;

pub mod stackmap;

pub fn readelf(path: &Path) -> Vec<u8> {
    let data = fs::read(path).unwrap();
    let elf = Elf::parse(&data).expect("Failed to parse ELF");

    let section = elf
        .section_headers
        .iter()
        .find(|section| elf.shdr_strtab.get_at(section.sh_name).unwrap_or("") == ".llvm_stackmaps")
        .unwrap();

    Vec::from(&data[section.sh_offset as usize..(section.sh_offset + section.sh_size) as usize])
}
