use anyhow::{Context, Result};
use clap::Parser;
use day3::CLIArgs;
fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let insts = day3::read_insts(&args.input_path).with_context(|| {
        format!(
            "Failed to read instructions from given file({}).",
            args.input_path.display()
        )
    })?;

    let sum = insts.iter().map(|i| i.mul_sum()).sum::<usize>();
    println!("The total sum of correct multiply instructions is {}.", sum);

    Ok(())
}
