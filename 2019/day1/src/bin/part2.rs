use day1::mass::read_md_mass;

fn main() {
    let mass_file = "inputs.txt";
    let mass_list: Vec<u32> =
        read_md_mass(mass_file).expect(&format!("Failed to read input file: {}", mass_file));
    let mod_count = mass_list.len();
    let fuel_sum: usize = mass_list.iter().map(|m: &u32| calc_mass_fuel(*m)).sum();
    print!(
        "There are {} modules in total, and {} units of fuel are needed.",
        mod_count, fuel_sum
    );
}

fn calc_mass_fuel(m: u32) -> usize {
    let mut fuel_sum = 0usize;
    let mut mass: isize = m.try_into().unwrap();
    loop {
        let fuel = mass / 3 - 2;
        if fuel <= 0 {
            break;
        }

        fuel_sum += usize::try_from(fuel).unwrap();
        mass = fuel;
    }

    fuel_sum
}

#[test]
fn test_calc_mass_fuel_zero_mass() {
    assert_eq!(calc_mass_fuel(0), 0)
}

#[test]
fn test_calc_mass_fuel_less_than_6_mass() {
    assert_eq!(calc_mass_fuel(1), 0);
    assert_eq!(calc_mass_fuel(2), 0);
    assert_eq!(calc_mass_fuel(3), 0);
    assert_eq!(calc_mass_fuel(4), 0);
    assert_eq!(calc_mass_fuel(5), 0);
    assert_eq!(calc_mass_fuel(6), 0);
}

#[test]
fn test_calc_mass_fuel_sample_mass() {
    assert_eq!(calc_mass_fuel(12), 2);
    assert_eq!(calc_mass_fuel(14), 2);
    assert_eq!(calc_mass_fuel(1969), 966);
    assert_eq!(calc_mass_fuel(100756), 50346);
}
