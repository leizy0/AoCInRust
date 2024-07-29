use anyhow::{Context, Result};
use clap::Parser;
use day23::{CLIArgs, CupGame};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let mut cup_game = CupGame::try_from(args.init_cups.as_str()).with_context(|| {
        format!(
            "Failed to initialize a cup game from given intial cup sequence: {}.",
            args.init_cups
        )
    })?;
    const MOVE_COUNT: usize = 100;

    for _ in 0..MOVE_COUNT {
        cup_game.one_move();
    }
    let cup_n = cup_game.cup_n();
    const START_CUP_ID: usize = 1;
    let mut seq_after_cup_id = String::new();
    let mut cur_cup = START_CUP_ID;
    for _ in 0..(cup_n - 1) {
        cur_cup = cup_game.next(cur_cup).unwrap();
        seq_after_cup_id.push(char::from_digit(cur_cup as u32, 10).unwrap());
    }
    println!(
        "After {} moves, the final cup sequence after cup {} is {}.",
        MOVE_COUNT, START_CUP_ID, seq_after_cup_id
    );

    Ok(())
}
