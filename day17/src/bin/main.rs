use day17::{map, Position};
use day17::sim::Simulator;

fn main() {
    let input_path = "input.txt";
    let und_map = map::load_und_map(input_path).expect(&format!("Failed to load underground map from input file({})", input_path));
    let vert_range = und_map.vert_range();
    let simulator = Simulator::new(und_map);
    match simulator.simulate(&Position::new(0, 500), &vert_range) {
        Ok(water_map) => {
            let (reach_water_count, rest_water_count) = water_map.count_water();
            let water_count = rest_water_count + reach_water_count;
            println!("In range({:?}), rest water(~) count = {}, reach water(|) count = {}, sum of two is {}",
                vert_range, rest_water_count, reach_water_count, water_count);
        }
        Err(e) => println!("Failed to simulate water permeation, get error({})", e),
    }
}
