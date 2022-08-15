use day24::{sim::Simulator, unit};

fn main() {
    let immune_input_path = "immune.txt";
    let infection_input_path = "infection.txt";
    let mut immune_sys = unit::load_army(immune_input_path).expect(&format!(
        "Failed to load immune system from input file({})",
        immune_input_path
    ));
    let mut infection_sys = unit::load_army(infection_input_path).expect(&format!(
        "Failed to load infection system from input file({})",
        infection_input_path
    ));
    let simulator = Simulator::new();
    match simulator.simulate(&mut [&mut immune_sys, &mut infection_sys]) {
        Ok(winner_ind) => {
            let winner = ["immune system", "infection system"][winner_ind];
            println!(
                "Final winner is {}, has {} units left.",
                winner,
                [&immune_sys, &infection_sys][winner_ind]
                    .groups()
                    .map(|g| g.count())
                    .sum::<usize>()
            );
        },
        Err(e) => println!("Failed to simulate army fight, get error({:?})", e),
    }
}
