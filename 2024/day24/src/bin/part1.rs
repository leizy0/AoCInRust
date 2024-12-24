use anyhow::{Context, Result};
use clap::Parser;
use day24::CLIArgs;

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (init_wires, mut circuit) =
        day24::read_circuit_info(&args.input_path).with_context(|| {
            format!(
                "Failed to read circuit information in given file({}).",
                args.input_path.display()
            )
        })?;

    circuit.init(&init_wires);
    if let Some(n) = circuit.simulate() {
        println!(
            "The result of simulation with the given circuit information is {}.",
            n
        );
    } else {
        eprintln!("The given circuit doesn't give output after simulation.");
    }

    Ok(())
}
