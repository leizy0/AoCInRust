use day18::Error;

fn main() -> Result<(), Error> {
    let input_path = "inputs.txt";
    let vault_map = day18::read_vault_map(input_path)?;
    let shortest_collect_path = day18::find_shortest_collect_path(&vault_map);
    println!(
        "The shortest path in given vault to collect keys has {} steps.",
        shortest_collect_path.len()
    );
    Ok(())
}
