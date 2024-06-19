use std::iter;

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
    jolts.sort_unstable();
    let device_jolt = if let Some(last_jolt) = jolts.last() {
        last_jolt + 3
    } else {
        eprintln!("Given empty list of joltage adapter ratings.");
        return;
    };

    let mut diff_counts = [0usize; 4];
    for (ind, (last, cur)) in iter::zip(
        iter::once(0).chain(jolts.iter().copied()),
        jolts.iter().copied().chain(iter::once(device_jolt)),
    )
    .enumerate()
    {
        let diff = cur - last;
        if diff >= diff_counts.len() {
            println!("There is a invalid difference of {} jolts([{}] = {}, [{}] = {}), expect differences which are smaller than {}.", diff, ind as isize - 1, last, ind, cur, diff_counts.len());
            return;
        }

        diff_counts[diff] += 1;
    }

    println!("The counts of differences(in range [0, {}]) are {:?}, so the product of counts which has 1 or 3 jolts difference is {}.", diff_counts.len() - 1, diff_counts, diff_counts[1] * diff_counts[3])
}
