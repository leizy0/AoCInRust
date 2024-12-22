use std::{
    collections::{HashMap, HashSet},
    iter,
};

use anyhow::{Context, Result};
use clap::Parser;
use day22::{CLIArgs, ChangeSeq};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let init_numbers = day22::read_init_numbers(&args.input_path).with_context(|| {
        format!(
            "Failed to read init numbers from given file({}).",
            args.input_path.display()
        )
    })?;

    let generations_n = 2000;
    const CHANGES_N_IN_SEQ: usize = 4;
    let mut changes_total_price = HashMap::new();
    for sec_n in init_numbers {
        let prices = iter::once(sec_n.n())
            .chain(sec_n.take(generations_n))
            .map(|n| n % 10)
            .collect::<Vec<_>>();
        let changes = prices[..(prices.len() - 1)]
            .iter()
            .zip(prices[1..].iter())
            .map(|(last_p, p)| isize::try_from(*p).unwrap() - isize::try_from(*last_p).unwrap())
            .collect::<Vec<_>>();
        let mut checked_change_seq = HashSet::new();
        for (slice_start_ind, change_slice) in changes.windows(CHANGES_N_IN_SEQ).enumerate() {
            let change_seq = ChangeSeq::<CHANGES_N_IN_SEQ>::try_from(change_slice).unwrap();
            if checked_change_seq.insert(change_seq.clone()) {
                let price = prices[slice_start_ind + CHANGES_N_IN_SEQ];
                *changes_total_price.entry(change_seq).or_insert(0) += price;
            }
        }
    }

    let (most_bananas_seq, most_bananas_n) = changes_total_price
        .iter()
        .max_by_key(|(_, price)| **price)
        .unwrap();
    println!(
        "With given data, one can get {} banana(s) at most by change sequence({}).",
        most_bananas_n, most_bananas_seq
    );

    Ok(())
}
