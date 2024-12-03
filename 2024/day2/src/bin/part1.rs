use anyhow::{Context, Result};
use clap::Parser;
use day2::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let reports = day2::read_reps(&args.input_path).with_context(|| {
        format!(
            "Failed to read reports from given file({}).",
            args.input_path.display()
        )
    })?;

    let safe_count = reports.iter().filter(|r| r.is_safe()).count();
    println!(
        "There is(are) {} safe report(s) in the whole reports list.",
        safe_count
    );

    Ok(())
}
