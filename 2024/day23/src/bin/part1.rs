use anyhow::{Context, Result};
use clap::Parser;
use day23::{CLIArgs, ComputerLinkGraph};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let links = day23::read_links(&args.input_path).with_context(|| {
        format!(
            "Failed to read computer links in given file({}).",
            args.input_path.display()
        )
    })?;

    let graph = ComputerLinkGraph::new(&links);
    let all_3_groups = graph.all_3_groups();
    let target_first_letter = 't';
    let candidate_groups_n = all_3_groups
        .iter()
        .filter(|group| {
            group
                .iter()
                .any(|computer| computer.name().starts_with(target_first_letter))
        })
        .count();
    println!(
        "The number of 3-groups that have at least one computer name starts with letter {} is {}.",
        target_first_letter, candidate_groups_n
    );

    Ok(())
}
