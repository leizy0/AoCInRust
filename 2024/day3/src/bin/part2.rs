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

    let sum = insts
        .iter()
        .fold((0, true), |(sum, do_mul), i| {
            let (cur_sum, cur_do_mul) = i.mul_sum_enable(true, do_mul);
            (sum + cur_sum, cur_do_mul)
        })
        .0;
    println!("The total sum of correct multiply instructions is {}.", sum);

    Ok(())
}
