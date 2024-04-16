use day20::{maze::RecursiveMaze, Error};

fn main() -> Result<(), Error> {
    let input_path = "inputs.txt";
    let start_gate_name = "AA";
    let stop_gate_name = "ZZ";
    let maze = day20::read_maze(input_path, start_gate_name, stop_gate_name)?;
    let maze = RecursiveMaze::new(maze);
    println!("Given maze looks like:\n{}", maze);
    let shortest_path = day20::find_shortest_path(&maze)?;
    println!(
        "It needs at least {} steps for going from AA to ZZ.",
        shortest_path.len()
    );
    Ok(())
}
