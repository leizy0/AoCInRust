use anyhow::{Context, Result};
use clap::Parser;
use day13::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let mut machines = day13::read_machines(&args.input_path).with_context(|| {
        format!(
            "Failed to read claw machines from given file({}).",
            args.input_path.display()
        )
    })?;

    let prize_offset = 10000000000000;
    machines.iter_mut().for_each(|machine| {
        machine.change_prize(|(x_prize, y_prize)| (x_prize + prize_offset, y_prize + prize_offset))
    });
    let min_tokens_sum = machines
        .iter()
        .map(|m| {
            m.solutions()
                .iter()
                .map(|solution| solution.tokens_n())
                .min()
                .unwrap_or(0)
        })
        .sum::<usize>();
    println!(
        "The sum of minimium tokens for solving given claw machines is {}.",
        min_tokens_sum
    );

    Ok(())
}
