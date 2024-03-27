use std::process;

use day14::{parse_reactions, Chemical};

fn main() {
    let input_path = "inputs.txt";
    let reaction_map = parse_reactions(input_path).expect(&format!(
        "Failed to parse reaction map from given input({}).",
        input_path
    ));

    let target_chemical = Chemical::new("FUEL");
    if !reaction_map.has(&target_chemical) {
        eprintln!("Can't find reaction outputs target chemical(FUEL) in given reactions.");
        process::exit(-1);
    }
    
    let ore_unit_n = 1000000000000u64;
    match reaction_map.synthesize(&target_chemical, ore_unit_n) {
        Ok((target_unit_n, left_unit_n)) => println!(
            "{} unit(s) of ORE can produce at most {} unit(s) {}, left {} units ORE",
            ore_unit_n,
            target_unit_n,
            &target_chemical,
            left_unit_n
        ),
        Err(e) => eprintln!(
            "Failed to synthesize given chemical({}) from ORE, get error({})",
            target_chemical,
            e
        ),
    }
}
