use std::{fs, path::Path};

use goblin::elf::Elf;

pub mod safepoint_gen;
pub mod stackmap;

pub fn read_reloc_names(path: &Path, section_name: &str) -> Vec<String> {
    let data = fs::read(path).unwrap();
    let elf = Elf::parse(&data).expect("Failed to parse ELF");

    let reloc_section = elf.shdr_relocs.iter().find(|(idx, _)| {
        elf.shdr_strtab
            .get_at(elf.section_headers[*idx].sh_name)
            .unwrap_or("")
            == section_name
    });

    let reloc_section = match reloc_section {
        Some(reloc_section) => &reloc_section.1,
        None => return vec![],
    };

    let mut reloc_names = vec![];
    for reloc in reloc_section {
        let sym = elf.syms.get(reloc.r_sym).unwrap();
        let name = String::from(elf.strtab.get_at(sym.st_name).unwrap());
        reloc_names.push(name);
    }

    reloc_names
}

pub fn read_section_bytes(path: &Path, section_name: &str) -> Result<Vec<u8>, String> {
    let data = fs::read(path).unwrap();
    let elf = Elf::parse(&data).map_err(|e| format!("Failed to parse ELF file: {e}"))?;

    let section = elf
        .section_headers
        .iter()
        .find(|section| elf.shdr_strtab.get_at(section.sh_name).unwrap_or("") == section_name);

    match section {
        Some(section) => Ok(Vec::from(
            &data[section.sh_offset as usize..(section.sh_offset + section.sh_size) as usize],
        )),
        None => Err(format!("Section {section_name} not found in ELF file")),
    }
}

pub fn read_section_syms(path: &Path, section_name: &str) -> Vec<String> {
    let data = fs::read(path).unwrap();
    let elf = Elf::parse(&data).expect("Failed to parse ELF");

    let section_idx = elf
        .section_headers
        .iter()
        .position(|section| elf.shdr_strtab.get_at(section.sh_name).unwrap_or("") == section_name);

    let section_idx = match section_idx {
        Some(section_idx) => section_idx,
        None => return vec![],
    };

    elf.syms
        .iter()
        .filter_map(|sym| {
            if sym.st_shndx == section_idx {
                Some(String::from(elf.strtab.get_at(sym.st_name).unwrap()))
            } else {
                None
            }
        })
        .collect()
}
