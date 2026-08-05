use std::{fs, path::Path};

use goblin::elf::Elf;

pub mod stackmap;

pub fn read_reloc_names(path: &Path, section_name: &str) -> Vec<String> {
    let data = fs::read(path).unwrap();
    let elf = Elf::parse(&data).expect("Failed to parse ELF");

    let reloc_section = &elf
        .shdr_relocs
        .iter()
        .find(|(idx, _)| {
            elf.shdr_strtab
                .get_at(elf.section_headers[*idx].sh_name)
                .unwrap_or("")
                == section_name
        })
        .unwrap()
        .1;

    let mut reloc_names = vec![];
    for reloc in reloc_section {
        let sym = elf.syms.get(reloc.r_sym).unwrap();
        let name = String::from(elf.strtab.get_at(sym.st_name).unwrap());
        reloc_names.push(name);
    }

    reloc_names
}

pub fn read_section_bytes(path: &Path, section_name: &str) -> Vec<u8> {
    let data = fs::read(path).unwrap();
    let elf = Elf::parse(&data).expect("Failed to parse ELF");

    let section = elf
        .section_headers
        .iter()
        .find(|section| elf.shdr_strtab.get_at(section.sh_name).unwrap_or("") == section_name)
        .unwrap();

    Vec::from(&data[section.sh_offset as usize..(section.sh_offset + section.sh_size) as usize])
}
