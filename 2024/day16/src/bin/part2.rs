use anyhow::{Context, Result};
use clap::Parser;
use day16::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let map = day16::read_map(&args.input_path).with_context(|| {
        format!(
            "Failed to read map from given file({}).",
            args.input_path.display()
        )
    })?;

    if let Some((action_graph, _)) = map.min_score_action_graph() {
        let min_score_path_pos_n = map.pos_n_on_graph(&action_graph);
        println!(
            "There are {} positions in map that's on at least one path with the minimium score.",
            min_score_path_pos_n
        );
    } else {
        eprintln!("There're no actions can complete the given map.");
    }

    Ok(())
}
