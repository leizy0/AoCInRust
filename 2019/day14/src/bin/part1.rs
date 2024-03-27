use std::process;

use day14::{parse_reactions, Chemical, Material};

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
    let target_unit_n = 1;
    let target_material = Material::new(target_chemical, target_unit_n);
    match reaction_map.decompose(&target_material) {
        Ok(u) => println!(
            "{} unit(s) of {} needs at least {} unit(s) ORE",
            target_unit_n,
            target_material.chemical(),
            u
        ),
        Err(e) => eprintln!(
            "Failed to decompose given chemical({}) to ORE, get error({})",
            target_material.chemical(),
            e
        ),
    }
}
