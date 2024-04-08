use day18::Error;

fn main() -> Result<(), Error> {
    let input_path = "inputs.txt";
    let mut vault_map = day18::read_vault_map(input_path)?;

    // Change map to match requirements in part 2.
    day18::split_entrance(&mut vault_map);
    let shortest_collect_path = day18::find_shortest_collect_path(&vault_map);
    let shortest_collect_path_steps_n: usize =
        shortest_collect_path.iter().map(|path| path.len()).sum();
    println!(
        "The shortest path in given vault(after entrance splited) to collect keys with {} collector(s) has {} steps.",
        vault_map.entrance_n(),
        shortest_collect_path_steps_n
    );
    Ok(())
}
