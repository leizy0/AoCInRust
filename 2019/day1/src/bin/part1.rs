use day1::mass::read_md_mass;

fn main() {
    let mass_file = "inputs.txt";
    let mass_list: Vec<u32> =
        read_md_mass(mass_file).expect(&format!("Failed to read input file: {}", mass_file));
    let mod_count = mass_list.len();
    let fuel_sum: u32 = mass_list.iter().map(|m: &u32| m / 3 - 2).sum();
    print!(
        "There are {} modules in total, and {} units of fuel are needed.",
        mod_count, fuel_sum
    );
}
