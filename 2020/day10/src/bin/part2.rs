use clap::Parser;
use day10::CLIArgs;

fn main() {
    let args = CLIArgs::parse();
    let mut jolts = day10::read_jolts_n(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read joltage adapter ratings from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    if jolts.is_empty() {
        eprintln!("Given empty list of joltage adapter ratings.");
        return;
    }
    jolts.sort_unstable();

    let jolts_n = jolts.len();
    let mut ways_to_dev = vec![0usize; jolts_n];
    ways_to_dev[jolts_n - 1] = 1;
    // Accumulate connection ways from rear to front.
    for ind in (1..(ways_to_dev.len())).rev() {
        let mut up_ind = ind - 1;
        while jolts[ind] - jolts[up_ind] <= 3 {
            // Accumulate to upper joltage adapter.
            ways_to_dev[up_ind] += ways_to_dev[ind];
            if up_ind == 0 {
                break;
            }
            up_ind -= 1;
        }
    }

    let ways_from_outlet = ways_to_dev
        .iter()
        .enumerate()
        .take_while(|(ind, _)| jolts[*ind] <= 3)
        .map(|(_, way_n)| way_n)
        .sum::<usize>();
    println!(
        "The count of ways from charging outlet to device(using given {} joltage adapters) are {}.",
        jolts.len(),
        ways_from_outlet
    );
}
