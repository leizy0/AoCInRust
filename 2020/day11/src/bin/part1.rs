use clap::Parser;
use day11::{CLIArgs, TileType};

fn main() {
    let args = CLIArgs::parse();
    let mut seat_map = day11::read_sm(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read map of seats layout from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();

    let mut step_count = 0;
    loop {
        let chg_count = seat_map.step();
        if chg_count == 0 {
            break;
        }

        step_count += 1;
    }

    println!("After {} step(s), given seats layout stabilizes, and there are {} seats have been occupied.", step_count, seat_map.count(TileType::Occupied));
}
