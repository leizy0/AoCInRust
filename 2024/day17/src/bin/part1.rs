use anyhow::{Context, Result};
use clap::Parser;
use day17::{CLIArgs, Computer};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (registers, code) = day17::read_debug_info(&args.input_path)
        .with_context(|| format!("failed to open given file({}).", args.input_path.display()))?;
    let mut computer = Computer::new(&registers);
    computer.run(&code)?;
    let output_str = computer
        .output()
        .iter()
        .map(|n| format!("{}", n))
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "After running given code, the computer output {}.",
        output_str
    );

    Ok(())
}
