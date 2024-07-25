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
    let mut mapped_names = ingrd_allrg_map
        .iter()
        .map(|(ingrd_id, allrg_id)| {
            (
                foods.ingrd_name(*ingrd_id).unwrap(),
                foods.allrg_name(*allrg_id).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    mapped_names.sort_unstable_by_key(|(_, allrg_name)| *allrg_name);
    let dangerous_ingrd_names = mapped_names
        .iter()
        .map(|(ingrd_name, _)| *ingrd_name)
        .collect::<Vec<_>>();
    println!(
        "The canonical dangerous ingredient list sorted alphabetically by their cotained allgery name is {}.",
        dangerous_ingrd_names.join(",")
    );

    Ok(())
}
