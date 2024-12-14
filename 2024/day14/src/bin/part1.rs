use anyhow::{Context, Result};
use clap::Parser;
use day14::{CLIArgs, Map};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let mut robots = day14::read_robots(&args.input_path).with_context(|| {
        format!(
            "Failed to read robots from given file({}).",
            args.input_path.display()
        )
    })?;

    let map = Map::new(args.map_width, args.map_height);
    let mut counts_in_quads = [0usize; 4];
    let move_count = 100;
    for r in &mut robots {
        r.move_n_in(move_count, &map);
        if let Some(quad_ind) = map.quad_ind(r.pos()) {
            counts_in_quads[quad_ind] += 1;
        }
    }
    println!(
        "The security factor(product of robots's count after moved in 4 quadrants) is {}.",
        counts_in_quads.iter().product::<usize>()
    );

    Ok(())
}
