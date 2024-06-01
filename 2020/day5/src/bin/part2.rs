use clap::Parser;
use day5::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let passes = day5::read_pass(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read boarding pass from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    let pass_ids = passes.iter().map(|p| p.id()).collect::<Vec<_>>();
    let min_id = *pass_ids.iter().min().unwrap();
    let max_id = *pass_ids.iter().max().unwrap();
    let mut on_list_flags = vec![false; max_id - min_id + 1];
    for id in &pass_ids {
        on_list_flags[id - min_id] = true;
    }

    let mut missing_id = None;
    for (ind, flag) in on_list_flags.iter().enumerate() {
        if !flag {
            let this_id = ind + min_id;
            if *missing_id.get_or_insert(this_id) != this_id {
                println!(
                    "There are at least two missing board pass ids({}, {}) in given list.",
                    missing_id.unwrap(),
                    this_id
                );
                return;
            }
        }
    }

    if let Some(missing_id) = missing_id {
        println!(
            "The only missing board pass id in given list is {}.",
            missing_id
        );
    } else {
        println!(
            "There isn't any missing board pass whose id is in range [{}, {}] in given list.",
            min_id, max_id
        );
    }
}
