use std::io::stdin;

use anyhow::{Context, Result};
use clap::Parser;
use day14::{Map, Part2CLIArgs};

fn main() -> Result<()> {
    let args = Part2CLIArgs::parse();
    let mut robots = day14::read_robots(&args.input_path).with_context(|| {
        format!(
            "Failed to read robots from given file({}).",
            args.input_path.display()
        )
    })?;

    let map = Map::new(args.map_width, args.map_height);
    let mut move_count = args.move_start;
    if move_count != 0 {
        for r in robots.iter_mut() {
            r.move_n_in(move_count, &map);
        }
    }

    let mut input = String::new();
    loop {
        for r in robots.iter_mut() {
            r.move_n_in(args.move_step, &map);
        }
        move_count += args.move_step;

        println!(
            "After {} move(s), with step({}):",
            move_count, args.move_step
        );
        map.display(&robots)?;
        stdin().read_line(&mut input)?;
        input = input.trim().to_string();
        if input == "e" || input == "exit" {
            break;
        }
    }

    Ok(())
}
