use anyhow::{Context, Result};
use clap::Parser;
use day1::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (list0, mut list1) = day1::read_lists(&args.input_path).with_context(|| {
        format!(
            "Failed to read location ID lists from given input file({}).",
            args.input_path.display()
        )
    })?;

    list1.sort_unstable();
    let mut sim_score = 0;
    for id0 in &list0 {
        if let Ok(id_ind) = list1.binary_search(id0) {
            let front_count = list1[..id_ind]
                .iter()
                .rev()
                .enumerate()
                .skip_while(|(_, id1)| *id1 == id0)
                .next()
                .map(|(n, _)| n)
                .unwrap_or(id_ind);
            let rear_count = list1[id_ind..]
                .iter()
                .enumerate()
                .skip_while(|(_, id1)| *id1 == id0)
                .next()
                .map(|(n, _)| n)
                .unwrap_or(list1.len() - id_ind);
            sim_score += id0 * (front_count + rear_count);
        }
    }
    println!(
        "The similarity score between two given list is {}.",
        sim_score
    );

    Ok(())
}
