use anyhow::{Context, Result};
use clap::Parser;
use day22::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let mut init_numbers = day22::read_init_numbers(&args.input_path).with_context(|| {
        format!(
            "Failed to read init numbers from given file({}).",
            args.input_path.display()
        )
    })?;

    let generations_n = 2000;
    let new_numbers_sum = init_numbers
        .iter_mut()
        .map(|n| n.nth(generations_n - 1).unwrap())
        .sum::<usize>();
    println!("The sum of secret numbers that are generated after {} generation(s) of given init numbers is {}.", generations_n, new_numbers_sum);

    Ok(())
}
