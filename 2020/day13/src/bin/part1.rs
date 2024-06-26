use clap::Parser;
use day13::CLIArgs;

fn main() {
    let args = CLIArgs::parse();
    let (depart_time, bus_cycles) = day13::read_notes(&args.input_path).inspect_err(|e| eprintln!("Failed to read notes about bus schedules from given input file({}), get error({}).", args.input_path.display(), e)).unwrap();
    if let Some((wait_time, bus_cycle)) = bus_cycles
        .iter()
        .map(|c| ((c - depart_time % c) % c, c))
        .min_by_key(|(wt, _)| *wt)
    {
        println!("After waiting {} minutes, the earliest bus({}) will arrive, so the product of waiting minutes and bus id is {}.", wait_time, bus_cycle, wait_time * bus_cycle);
    } else {
        println!("There's no bus can be taken from given schedules.");
    }
}
