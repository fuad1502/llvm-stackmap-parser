use std::path::PathBuf;

use llvm_stackmap_parser::{readelf, stackmap::StackMap};

fn main() {
    let path = PathBuf::from("/home/fuad1502/code/oonta/ocaml/merge_sort.o");
    let bytes = readelf(&path);
    let stack_map = StackMap::from(&bytes[..]);

    println!("{:#?}", stack_map);
}
