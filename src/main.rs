use std::{env, path::PathBuf};

use llvm_stackmap_parser::{
    read_reloc_names, read_section_bytes, read_section_syms, safepoint_gen::gen_safepoints_source,
    stackmap::StackMap,
};

fn main() {
    let path = PathBuf::from("/home/fuad1502/code/oonta/ocaml/merge_sort.o");
    let bytes = read_section_bytes(&path, ".llvm_stackmaps");
    let stack_map = StackMap::from(&bytes[..]);
    let reloc_names = read_reloc_names(&path, ".rela.llvm_stackmaps");
    let global_gcroot_names = read_section_syms(&path, ".gcroots");

    gen_safepoints_source(
        &stack_map,
        &reloc_names,
        &global_gcroot_names,
        &env::current_dir().unwrap(),
    )
    .unwrap();
}
