use anyhow::{Context, Result};
use clap::Parser;
use day25::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (card_pub_key, door_pub_key) =
        day25::read_pub_keys(&args.input_path).with_context(|| {
            format!(
                "Failed to read public keys from given input file: {}.",
                args.input_path.display()
            )
        })?;

    const SUBJECT_NUMBER: usize = 7;
    const DIVISOR: usize = 20201227;
    let card_loop_size = day25::find_loop_size(card_pub_key, SUBJECT_NUMBER, DIVISOR);
    let door_loop_size = day25::find_loop_size(door_pub_key, SUBJECT_NUMBER, DIVISOR);
    let card_encryp_key = day25::transform_subject_n(door_pub_key, DIVISOR, card_loop_size);
    let door_encryp_key = day25::transform_subject_n(card_pub_key, DIVISOR, door_loop_size);
    if card_encryp_key != door_encryp_key {
        eprintln!(
            "The encryption key used by door({}) and card({}) aren't the same as designed.",
            door_encryp_key, card_encryp_key
        );
    } else {
        println!(
            "The encryption key used by door and card is {}.",
            card_encryp_key
        );
    }

    Ok(())
}
