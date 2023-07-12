use day24::{sim::{Simulator, Cheat}, unit};

fn main() {
    let immune_input_path = "immune.txt";
    let infection_input_path = "infection.txt";
    let immune_sys = unit::load_army(immune_input_path).expect(&format!(
        "Failed to load immune system from input file({})",
        immune_input_path
    ));
    let infection_sys = unit::load_army(infection_input_path).expect(&format!(
        "Failed to load infection system from input file({})",
        infection_input_path
    ));

    let mut min_boost = 0;
    let mut max_boost = 1000;
    loop {
        let cur_boost = (min_boost + max_boost) / 2;
        println!("Try boost {} in range({}, {})", cur_boost, min_boost, max_boost);

        let mut immune_test_system = immune_sys.clone();
        let mut infection_test_system = infection_sys.clone();
        let simulator = Simulator::with_cheat(Cheat::ArmyAttackBoost{army_ind: 0, boost_point: cur_boost});
        let res = simulator.simulate(&mut [&mut immune_test_system, &mut infection_test_system]);
        if res.is_ok() && res.unwrap() == 0 {
            max_boost = cur_boost;
        } else {
            min_boost = cur_boost + 1;
        }

        if min_boost >= max_boost {
            let immune_left_unit_count = immune_test_system
                .groups()
                .map(|g| g.count())
                .sum::<usize>();
            println!("Immune system can win with the most modest attack boost({}), and left {} unit(s)", min_boost, immune_left_unit_count);
            break;
        }
    }
}
