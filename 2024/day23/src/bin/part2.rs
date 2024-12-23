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
    let max_all_connected_groups = graph.max_all_connected_groups();
    match max_all_connected_groups.len() {
        0 => eprintln!("There's no link in given computer link graph."),
        1 => {
            let mut max_members_n_group = max_all_connected_groups.iter().next().unwrap().clone();
            max_members_n_group.sort_unstable();
            let password = max_members_n_group
                .iter()
                .map(|computer| computer.name())
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "The password for the largest all connected group is {}.",
                password,
            );
        }
        count => eprintln!(
            "There are {} groups which has the maximum member count({}), not expected only one.",
            count,
            max_all_connected_groups.iter().next().unwrap().len()
        ),
    }

    Ok(())
}
