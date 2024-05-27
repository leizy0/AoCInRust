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
    let tree_count = map
        .tiles_on_ray(&Position::new(0, 0), &dir)
        .filter(|tt| **tt == TileType::Tree)
        .count();
    println!("There are {} trees if following {:?}.", tree_count, dir);
}
