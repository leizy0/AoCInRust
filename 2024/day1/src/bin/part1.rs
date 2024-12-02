use anyhow::{Context, Result};
use clap::Parser;
use day1::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (mut list0, mut list1) = day1::read_lists(&args.input_path).with_context(|| {
        format!(
            "Failed to read location ID lists from given input file({}).",
            args.input_path.display()
        )
    })?;

    list0.sort_unstable();
    list1.sort_unstable();
    let diff_sum = list0
        .iter()
        .zip(list1.iter())
        .map(|(id0, id1)| id0.abs_diff(*id1))
        .sum::<usize>();
    println!(
        "The total sum of differences between two given lists after sorted is {}.",
        diff_sum
    );

    Ok(())
}
