use anyhow::{Context, Result};
use clap::Parser;
use day7::{CLIArgs, Operator};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let equations = day7::read_equations(&args.input_path).with_context(|| {
        format!(
            "Failed to read equations from given file({}).",
            args.input_path.display()
        )
    })?;

    let sum = equations
        .iter()
        .filter(|e| e.is_possible(&[Operator::Plus, Operator::Multiply, Operator::Concatenation]))
        .map(|e| e.result())
        .sum::<usize>();
    println!("The sum of possible equations' results is {}.", sum);

    Ok(())
}
