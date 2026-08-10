use std::io::Write;
use std::process::Command;
use std::{fs::File, io::BufWriter};

use crate::stackmap::{LocationType, StackMap};

struct SafepointSourceGenerator<'a> {
    wr: BufWriter<File>,
    stack_map: &'a StackMap,
    reloc_names: &'a [String],
}

pub fn gen_safepoints_source(stack_map: &StackMap, reloc_names: &[String]) -> Result<(), String> {
    let file = File::create("safepoints.c").map_err(|e| e.to_string())?;
    let wr = BufWriter::new(file);

    let mut generator = SafepointSourceGenerator {
        wr,
        stack_map,
        reloc_names,
    };

    generator.gen_header().map_err(|e| e.to_string())?;
    generator.gen_externs().map_err(|e| e.to_string())?;
    generator.gen_offsets().map_err(|e| e.to_string())?;
    generator.gen_safepoints().map_err(|e| e.to_string())?;

    generator.wr.flush().map_err(|e| e.to_string())?;
    gen_header_file()?;
    generator.compile_safepoints()?;
    generator.archive_safepoints()?;

    std::fs::remove_file("safepoints.o").map_err(|e| e.to_string())?;
    std::fs::remove_file("safepoints.c").map_err(|e| e.to_string())?;

    Ok(())
}

impl<'a> SafepointSourceGenerator<'a> {
    fn gen_header(&mut self) -> Result<(), std::io::Error> {
        writeln!(self.wr, "#include \"safepoints.h\"")?;
        writeln!(self.wr)
    }

    fn gen_externs(&mut self) -> Result<(), std::io::Error> {
        for name in self.reloc_names {
            if name == "main" {
                writeln!(self.wr, "extern int main();")?
            } else {
                writeln!(
                    self.wr,
                    "extern char {}[] __asm__(\"{name}\");",
                    transform_name(name)
                )?
            }
        }
        writeln!(self.wr)
    }

    fn gen_offsets(&mut self) -> Result<(), std::io::Error> {
        for (i, record) in self.stack_map.stack_map_records.iter().enumerate() {
            for location in record.locations.iter().take(3) {
                if !matches!(location.typ, LocationType::Constant(0)) {
                    panic!("Whops!")
                }
            }
            write!(self.wr, "static uint64_t offsets_{i}[] = {{")?;
            for location in record.locations.iter().skip(3) {
                match location.typ {
                    LocationType::Indirect(7, offset) => write!(self.wr, "{offset}, ")?,
                    _ => panic!("Whoops!"),
                }
            }
            writeln!(self.wr, "}};")?;
        }
        writeln!(self.wr)
    }

    fn gen_safepoints(&mut self) -> Result<(), std::io::Error> {
        writeln!(self.wr, "struct Safepoint safepoints[] = {{")?;
        let mut curr_fun_idx = 0;
        let mut next_fun_record_idx = self.stack_map.stack_size_records[curr_fun_idx].record_count;
        for (i, record) in self.stack_map.stack_map_records.iter().enumerate() {
            if i == next_fun_record_idx as usize {
                curr_fun_idx += 1;
                next_fun_record_idx += self.stack_map.stack_size_records[curr_fun_idx].record_count;
            }
            write!(self.wr, "    {{")?;
            write!(
                self.wr,
                "(void *){} + {}, ",
                transform_name(&self.reloc_names[curr_fun_idx]),
                record.instruction_offset
            )?;
            write!(
                self.wr,
                "{}, ",
                self.stack_map.stack_size_records[curr_fun_idx].stack_size
            )?;
            write!(self.wr, "offsets_{}", i)?;
            writeln!(self.wr, "}},")?;
        }
        writeln!(self.wr, "}};")
    }

    fn compile_safepoints(&mut self) -> Result<(), String> {
        let mut cmd = Command::new("gcc");
        cmd.args(["-c", "safepoints.c", "-I", ".", "-o", "safepoints.o"]);
        execute_command(cmd)?;
        Ok(())
    }

    fn archive_safepoints(&mut self) -> Result<(), String> {
        let mut cmd = Command::new("ar");
        cmd.args(["rcs", "safepoints.a", "safepoints.o"]);
        execute_command(cmd)?;
        Ok(())
    }
}

fn gen_header_file() -> Result<(), String> {
    let file = File::create("safepoints.h").map_err(|e| e.to_string())?;
    let mut wr = BufWriter::new(file);
    write!(
        &mut wr,
        r#"#ifndef __LLVM_STACK_MAP_H
#define __LLVM_STACK_MAP_H

#include <stdint.h>

struct Safepoint {{
  void *location;
  uint64_t stack_size;
  uint64_t *obj_stack_offsets;
}};

extern struct Safepoint safepoints[];

#endif // __LLVM_STACK_MAP_H
"#
    )
    .map_err(|e| e.to_string())
}

fn transform_name(name: &str) -> String {
    name.replace(".", "_")
}

fn execute_command(mut cmd: Command) -> Result<(), String> {
    let error_message = format!("Error: failed to execute command ({cmd:?})");
    let output = cmd.output().map_err(|e| format!("{error_message}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{error_message}:\nStdout:\n{}Stderr:\n{}",
            str::from_utf8(&output.stdout).unwrap(),
            str::from_utf8(&output.stderr).unwrap()
        ));
    }
    Ok(())
}
