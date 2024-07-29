use anyhow::{Context, Result};
use clap::Parser;
use day23::{CLIArgs, CupGame, Error};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let init_seq = args
        .init_cups
        .chars()
        .map(|c| {
            c.to_digit(10)
                .map(|n| n as usize)
                .ok_or(Error::InvalidCupChar(c))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut cup_game = CupGame::try_from_seq(init_seq.len(), init_seq.iter().copied())
        .with_context(|| {
            format!(
                "Failed to initialize a cup game from given intial cup sequence: {}.",
                args.init_cups
            )
        })?;
    const MOVE_COUNT: usize = 100;

    for _ in 0..MOVE_COUNT {
        cup_game.one_move();
    }
    let cups_n = cup_game.cups_n();
    const START_CUP_ID: usize = 1;
    let mut seq_after_cup_id = String::new();
    let mut cur_cup = START_CUP_ID;
    for _ in 0..(cups_n - 1) {
        cur_cup = cup_game.next(cur_cup).unwrap();
        seq_after_cup_id.push(char::from_digit(cur_cup as u32, 10).unwrap());
    }
    println!(
        "After {} moves, the final cup sequence after cup {} is {}.",
        MOVE_COUNT, START_CUP_ID, seq_after_cup_id
    );

    Ok(())
}
