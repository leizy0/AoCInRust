use anyhow::{Context, Result};
use clap::Parser;
use day21::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let foods = day21::read_foods(&args.input_path).with_context(|| {
        format!(
            "Failed to read information of foods from given input file({}).",
            args.input_path.display()
        )
    })?;

    let ingrd_allrg_map = day21::find_ingrd_allg_map(&foods);
    let uncertain_ingrd_appr_count = foods
        .iter()
        .map(|food| {
            food.ingrd_ids()
                .filter(|ingrd_id| !ingrd_allrg_map.contains_key(ingrd_id))
                .count()
        })
        .sum::<usize>();
    println!(
        "The count of appearance of the ingrediant which can't find any allergery is {}.",
        uncertain_ingrd_appr_count
    );

    Ok(())
}
