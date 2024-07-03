use clap::Parser;
use day15::{CLIArgs, MemGame};

fn main() {
    let args = CLIArgs::parse();
    let starting_nums = day15::read_nums(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read starging numbers from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let mut game = MemGame::new(&starting_nums);
    const TARGET_TURN: usize = 2020;

    println!(
        "Given starting numbers({:?}), the {}th number spoken in the game is {}.",
        starting_nums,
        TARGET_TURN,
        game.nth(TARGET_TURN - 1)
    )
}
