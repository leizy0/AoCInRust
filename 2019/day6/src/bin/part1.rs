use day6::orbit::read_orbits;

fn main() {
    let input_file = "inputs.txt";
    let orbit_tree = read_orbits(input_file).expect(&format!(
        "Failed to read orbit tree from file({})",
        input_file
    ));
    let total_orbit_count: usize = orbit_tree
        .obj_iter()
        .map(|o| orbit_tree.orbit_count_of(o).unwrap())
        .sum();
    println!("Total orbit count of given input is {}", total_orbit_count);
}
