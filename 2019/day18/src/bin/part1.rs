use day18::Error;

fn main() -> Result<(), Error> {
    let input_path = "inputs.txt";
    let vault_map = day18::read_vault_map(input_path)?;
    let shortest_collect_path = day18::find_shortest_collect_path(&vault_map);
    let shortest_collect_path_steps_n: usize =
        shortest_collect_path.iter().map(|path| path.len()).sum();
    println!(
        "The shortest path in given vault to collect keys with {} collector(s) has {} steps.",
        vault_map.entrance_n(),
        shortest_collect_path_steps_n
    );
    Ok(())
}
