use std::process::ExitCode;

use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;
use yield_lang::{parse, LLVMCodeGen};

#[derive(clap::Parser)]
struct Cli {
    input: PathBuf,
}

fn main() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();

    let module_name = cli
        .input
        .file_stem()
        .and_then(|s| s.to_str())
        .context("Failed get module name")?;

    let binding = inkwell::context::Context::create();
    let compiler = LLVMCodeGen::new(&binding, module_name);
    compiler.define_external_functions();

    let input = std::fs::read_to_string(cli.input.clone()).context("Failed to read input file")?;
    let ast = parse(&input).context("Failed to parse program")?;

    compiler.compile(ast).context("Failed to compile program")?;

    compiler
        .print_to_file(format!("{}.ll", module_name).as_str())
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to compile to output.ll")?;

    Ok(ExitCode::SUCCESS)
}
