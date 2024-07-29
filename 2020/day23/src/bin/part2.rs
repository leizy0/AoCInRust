use std::iter;

use anyhow::{Context, Result};
use clap::Parser;
use day23::{CLIArgs, CupGame};

fn main() -> Result<()> {
    const CUPS_N: usize = 1000000;
    let args = CLIArgs::parse();
    let init_seq_start = args
        .init_cups
        .chars()
        .map(|c| {
            c.to_digit(10)
                .map(|n| n as usize)
                .ok_or(day23::Error::InvalidCupChar(c))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let init_left_seq_start = init_seq_start.iter().max().copied().unwrap() + 1;
    let init_seq_iter = init_seq_start
        .iter()
        .copied()
        .chain(iter::successors(Some(init_left_seq_start), |n| {
            Some(n + 1).filter(|n| *n <= CUPS_N)
        }));
    let mut cup_game = CupGame::try_from_seq(CUPS_N, init_seq_iter).with_context(|| {
        format!(
            "Failed to initialize a cup game from given intial cup sequence: {}.",
            args.init_cups
        )
    })?;

    const MOVE_COUNT: usize = 10000000;
    for _ in 0..MOVE_COUNT {
        cup_game.one_move();
    }
    const START_CUP_ID: usize = 1;
    let star_cup_1 = cup_game.next(START_CUP_ID).unwrap();
    let star_cup_2 = cup_game.next(star_cup_1).unwrap();
    println!(
        "After {} moves, two cups hide stars are {} and {}(end up immediately clockwise of cup {}), so the product is {}.",
        MOVE_COUNT, star_cup_1, star_cup_2, START_CUP_ID, star_cup_1 * star_cup_2
    );

    Ok(())
}
