use anyhow::{Context, Result};
use clap::Parser;
use day4::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let letter_mat = day4::read_letter_mat(&args.input_path).with_context(|| {
        format!(
            "Failed to read letter matrix from given file({}).",
            args.input_path.display()
        )
    })?;

    let word = "XMAS";
    let word_count = letter_mat.search(word);
    println!(
        "Given word({}) appears {} time(s) in given letter matrix.",
        word, word_count
    );

    Ok(())
}
