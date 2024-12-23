use anyhow::{Context, Result};
use clap::Parser;
use day21::{CLIArgs, Keypad, Robot, UI};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let door_codes = day21::read_door_codes(&args.input_path).with_context(|| {
        format!(
            "Failed to read door codes from given file({}).",
            args.input_path.display()
        )
    })?;

    let robot_in_depressurized_space = Robot::new(Keypad::new_numeric());
    let robot_in_radiation = Robot::new(robot_in_depressurized_space);
    let robot_in_low_temperature = Robot::new(robot_in_radiation);

    let sum_of_complexities = door_codes
        .iter()
        .map(|code| robot_in_low_temperature.input(code.text()).unwrap()[0].len() * code.number())
        .sum::<usize>();
    println!(
        "The sum of complexities of given door codes is {}.",
        sum_of_complexities
    );

    Ok(())
}
