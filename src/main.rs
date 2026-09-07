use std::process::ExitCode;
use std::{env, path::PathBuf};

use llvm_stackmap_parser::{
    read_reloc_names, read_section_bytes, read_section_syms, safepoint_gen::gen_safepoints_lib,
    stackmap::StackMap,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!(
            r#"LLVM Stack Map parser

Usage: llvm-stackmap-parser <Object File>"#
        );
        return ExitCode::FAILURE;
    }

    let path = PathBuf::from(&args[1]);
    if !path.exists() {
        eprintln!("File {} not found", path.to_string_lossy());
        return ExitCode::FAILURE;
    }

    let bytes = match read_section_bytes(&path, ".llvm_stackmaps") {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };

    let stack_map = StackMap::from(&bytes[..]);

    let stack_map_relocs_names = read_reloc_names(&path, ".rela.llvm_stackmaps");
    if stack_map.num_functions as usize != stack_map_relocs_names.len() {
        eprintln!(
            "Number of relocation names in .rela.llvm_stackmaps does not match the number of functions in the parsed Stack Map"
        );
        return ExitCode::FAILURE;
    }

    let global_gcroot_names = read_section_syms(&path, ".gcroots");

    match gen_safepoints_lib(
        &stack_map,
        &stack_map_relocs_names,
        &global_gcroot_names,
        &env::current_dir().unwrap(),
    ) {
        Ok(path) => println!("Generated {}", path.to_string_lossy()),
        Err(e) => {
            eprintln!("Failed to generate safepoints lib: {e}");
            return ExitCode::FAILURE;
        }
    };

    ExitCode::SUCCESS
}
