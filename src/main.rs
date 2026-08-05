use std::path::PathBuf;

use llvm_stackmap_parser::{read_reloc_names, read_section_bytes, stackmap::StackMap};

fn main() {
    let path = PathBuf::from("/home/fuad1502/code/oonta/ocaml/merge_sort.o");
    let bytes = read_section_bytes(&path, ".llvm_stackmaps");
    let stack_map = StackMap::from(&bytes[..]);

    println!("{:#?}", stack_map);

    println!("{:#?}", read_reloc_names(&path, ".rela.llvm_stackmaps"));
}
