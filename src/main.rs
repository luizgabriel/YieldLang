use std::{env, process::ExitCode};

use anyhow::Context;
use clap::Parser;
use std::path::PathBuf;
use yield_lang::{parse, LLVMCodeGen};

#[derive(clap::Parser)]
struct Cli {
    input: PathBuf,

    #[arg(short, long)]
    output: Option<PathBuf>,

    #[arg(short, long)]
    debug: bool,
}

fn main() -> anyhow::Result<ExitCode> {
    let cli = Cli::parse();

    let input_stem = cli.input.file_stem().context("Failed to input stem")?;

    let module_name = input_stem
        .to_str()
        .context("Failed to convert input stem")?;

    let binding = inkwell::context::Context::create();
    let compiler = LLVMCodeGen::new(&binding, module_name);
    compiler.define_external_functions();

    let input = std::fs::read_to_string(cli.input.clone()).context("Failed to read input file")?;
    let ast = parse(&input).context("Failed to parse program")?;

    if cli.debug {
        println!("{:#?}", ast);
    }

    compiler.compile(ast).context("Failed to compile program")?;

    let current_dir = env::current_dir().context("Failed to get current directory")?;

    let output = cli.output.unwrap_or_else(|| {
        let out = current_dir.clone().join(input_stem);
        out.with_extension("ll");
        out
    });

    compiler
        .print_to_file(output.to_str().context("Failed to convert output path")?)
        .map_err(|e| anyhow::anyhow!(e))
        .context("Failed to compile to output file")?;

    Ok(ExitCode::SUCCESS)
}
