mod sim;
use std::env;

use sim::{Cheat, SimResult, Simulator, UnitRace};

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
    let mut cheat_attack_min = 3;
    let mut cheat_attack_max = 50;
    let mut cheat_ind = 0;
    while cheat_attack_min < cheat_attack_max {
        println!("Cheat#{}:", cheat_ind);
        let cur_cheat_attack = (cheat_attack_max + cheat_attack_min) / 2;
        let mut cur_simulator = simulator.clone();
        cur_simulator.add_cheat(Cheat::SetElfAttack {
            attack: cur_cheat_attack,
        });
        let cur_result = cur_simulator.simulate_with_cond(|units| {
            if units
                .iter()
                .filter(|u| u.race() == UnitRace::Elf)
                .any(|u| u.is_dead())
            {
                Some(UnitRace::Goblin)
            } else if units
                .iter()
                .filter(|u| u.race() == UnitRace::Goblin)
                .all(|u| u.is_dead())
            {
                Some(UnitRace::Elf)
            } else {
                None
            }
        });

        match cur_result {
            Ok(SimResult {
                winner,
                round_count,
                units,
            }) => {
                match winner {
                    UnitRace::Elf => {
                        cheat_attack_max = cur_cheat_attack;
                    }
                    UnitRace::Goblin => {
                        cheat_attack_min = cur_cheat_attack + 1;
                    }
                }

                let living_units_health_sum = units
                    .iter()
                    .filter(|u| !u.is_dead())
                    .map(|u| u.health())
                    .sum::<i32>();
                let prod = round_count * u32::try_from(living_units_health_sum).unwrap();
                println!("Final winner is {}, current elf's attack is {}, cost {} rounds, and product is {}", 
                    winner, cur_cheat_attack, round_count, prod);
            }
            Err(e) => println!("Failed to simulate given input settings, get error({})", e),
        }
        cheat_ind += 1;
        println!();
    }

    println!(
        "You can set elf's attack to {} to win goblins with the most modest cheat",
        cheat_attack_min
    );
}
