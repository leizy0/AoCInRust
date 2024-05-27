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

    let dirs = [
        Direction::new(1, 1),
        Direction::new(1, 3),
        Direction::new(1, 5),
        Direction::new(1, 7),
        Direction::new(2, 1),
    ];
    let tree_counts = dirs
        .iter()
        .map(|dir| {
            map.tiles_on_ray(&Position::new(0, 0), dir)
                .filter(|tt| **tt == TileType::Tree)
                .count()
        })
        .collect::<Vec<_>>();
    println!("Along given directions, the numbers of trees encountered are {:?}, and the product of these numbers is {}.", tree_counts, tree_counts.iter().product::<usize>());
}
