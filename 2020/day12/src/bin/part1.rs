use clap::Parser;
use day12::{CLIArgs, Ship};

fn main() {
    let args = CLIArgs::parse();
    let insts = day12::read_insts(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read instructions from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let mut ship = Ship::new();
    let start_pos = ship.pos();
    for inst in &insts {
        match ship.handle(inst) {
            Ok(_) => (),
            Err(e) => {
                eprintln!(
                    "Failed to handle given instruction({:?}), get error({})",
                    inst, e
                );
                return;
            }
        }
    }

    let cur_pos = ship.pos();
    println!("After {} instruction(s), ship moved from {} to {}, the Manhattan distance between these two locations is {}.", insts.len(), start_pos, cur_pos, cur_pos.m_dist(&start_pos));
}
