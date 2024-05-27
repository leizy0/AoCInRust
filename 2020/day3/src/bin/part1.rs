use clap::Parser;
use day3::{CliArgs, Direction, Position, TileType};

fn main() {
    let args = CliArgs::parse();
    let map = day3::read_map(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read map from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let dir = Direction::new(1, 3);
    let mut tree_count = 0;
    let mut cur_pos = Position::new(0, 0);
    while let Some(tt) = map.tile(&cur_pos) {
        if *tt == TileType::Tree {
            tree_count += 1;
        }

        cur_pos += &dir;
    }
    println!("There are {} trees if following {:?}.", tree_count, dir);
}
