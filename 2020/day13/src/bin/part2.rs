use clap::Parser;
use day13::CLIArgs;

fn main() {
    let args = CLIArgs::parse();
    let (_, bus_schedules) = day13::read_notes(&args.input_path).inspect_err(|e| eprintln!("Failed to read notes about bus schedules from given input file({}), get error({}).", args.input_path.display(), e)).unwrap();
    let mut t = 1;
    let mut interval = 1;
    for (bus_cycle, bus_interval) in bus_schedules {
        while (t + bus_interval) % bus_cycle != 0 {
            t += interval;
        }

        interval = day13::lcm(interval, bus_cycle);
    }

    println!(
        "The earliest timestamp that all buses depart properly is {}.",
        t
    );
}
