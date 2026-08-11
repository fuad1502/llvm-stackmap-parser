use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs::File, io::BufWriter};

use crate::stackmap::{LocationType, StackMap};

struct SafepointSourceGenerator<'a> {
    wr: BufWriter<File>,
    stack_map: &'a StackMap,
    reloc_names: &'a [String],
}

pub fn gen_safepoints_source(
    stack_map: &StackMap,
    reloc_names: &[String],
    output_dir: &Path,
) -> Result<PathBuf, String> {
    let mut safepoints_source = PathBuf::from(output_dir);
    safepoints_source.push("safepoints.c");

    let file = File::create(&safepoints_source).map_err(|e| e.to_string())?;
    let wr = BufWriter::new(file);

    let mut generator = SafepointSourceGenerator {
        wr,
        stack_map,
        reloc_names,
    };

    generator.gen_header().map_err(|e| e.to_string())?;
    generator.gen_safepoints_len().map_err(|e| e.to_string())?;
    generator.gen_externs().map_err(|e| e.to_string())?;
    generator.gen_offsets().map_err(|e| e.to_string())?;
    generator.gen_safepoints().map_err(|e| e.to_string())?;
    generator.wr.flush().map_err(|e| e.to_string())?;

    let safepoints_obj = generator.compile_safepoints(&safepoints_source)?;
    let safepoints_ar = generator.archive_safepoints(&safepoints_obj)?;

    std::fs::remove_file(safepoints_obj).map_err(|e| e.to_string())?;
    std::fs::remove_file(safepoints_source).map_err(|e| e.to_string())?;

    Ok(safepoints_ar)
}

impl<'a> SafepointSourceGenerator<'a> {
    fn gen_safepoints_len(&mut self) -> Result<(), std::io::Error> {
        writeln!(
            self.wr,
            "int safepoints_len = {};",
            self.stack_map.num_records
        )?;
        writeln!(self.wr)
    }

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

    fn compile_safepoints(&mut self, safepoints_source: &Path) -> Result<PathBuf, String> {
        let safepoints_obj = safepoints_source.with_extension("o");
        let mut include_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        include_dir.push("src");
        let mut cmd = Command::new("gcc");
        cmd.args([
            "-c",
            safepoints_source.to_str().unwrap(),
            "-I",
            include_dir.to_str().unwrap(),
            "-o",
            safepoints_obj.to_str().unwrap(),
        ]);
        execute_command(cmd)?;
        Ok(safepoints_obj)
    }

    fn archive_safepoints(&mut self, safepoints_obj: &Path) -> Result<PathBuf, String> {
        let safepoints_ar = safepoints_obj
            .with_file_name(
                String::from("lib") + safepoints_obj.file_name().unwrap().to_str().unwrap(),
            )
            .with_extension("a");
        let mut cmd = Command::new("ar");
        cmd.args([
            "rcs",
            safepoints_ar.to_str().unwrap(),
            safepoints_obj.to_str().unwrap(),
        ]);
        execute_command(cmd)?;
        Ok(safepoints_ar)
    }
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
