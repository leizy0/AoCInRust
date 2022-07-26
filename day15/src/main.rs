mod sim;
use std::env;

use sim::{SimResult, Simulator};

fn main() {
    let args: Vec<String> = env::args().collect();
    let input_path = if args.len() > 1 {
        &args[1]
    } else {
        "input.txt"
    };
    let (map, units) = sim::load_settings(input_path).expect(&format!(
        "Failed to load map from input file({})",
        input_path
    ));
    
    let simulator = Simulator::new(map, units);
    let res = simulator.simulate();
    match res {
        Ok(SimResult {
            winner,
            round_count,
            units,
        }) => {
            let living_units_health_sum = units
                .iter()
                .filter(|u| !u.is_dead())
                .map(|u| u.health())
                .sum::<i32>();
            let prod = round_count * u32::try_from(living_units_health_sum).unwrap();
            if let Some(winner) = winner {
                println!(
                    "Final winner is {}, cost {} rounds, and product is {}",
                    winner, round_count, prod
                );
            } else {
                println!(
                    "It's draw, cost {} rounds, and product is {}",
                    round_count, prod
                );
            }
        }
        Err(e) => println!("Failed to simulate given input settings, get error({})", e),
    }
}
