use day18::map;
use day18::sim::Simulator;

fn main() {
    let input_path = "input.txt";
    let map = map::load_lumber_map(input_path).expect(&format!(
        "Failed to load lumber map from input file({})",
        input_path
    ));
    let mut simulator = Simulator::new(map);
    let sim_tick_count_1 = 10; // Part 1
    let sim_tick_count_2 = 1000000000; // Part 2
    let sim_tick_count = sim_tick_count_2;
    match simulator.simulate(sim_tick_count) {
        Ok((tree_count, lumberyard_count)) => {
            let prod = tree_count * lumberyard_count;
            println!("After {} tick(s), there are {} wooded(tree) acres and {} lumberyards, total resource value(product) is {}",
                sim_tick_count_1, tree_count, lumberyard_count, prod);
        }
        Err(e) => println!("Failed to simulate this lumber game, get error({})", e),
    }
}
