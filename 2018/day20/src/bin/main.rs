use day20::map::{self, Position};

fn main() {
    let input_path = "input.txt";
    let map = map::load_map(input_path).expect(&format!(
        "Failed to load map from input file({})",
        input_path
    ));
    // log_map(&map);

    let neighbor_mat = map.neighbor_mat();
    let origin = Position::new(0, 0);
    let shortest_paths = map::bfs(&origin, &neighbor_mat);
    // Part 1
    let node_count_longest_path = shortest_paths
        .iter()
        .map(|(_, path)| path.len())
        .max()
        .expect(&format!("Found no path in map({:?})", map));
    // Part 2
    let min_node_count = 1000;
    let node_count_above_min_path_count = shortest_paths
        .iter()
        .map(|(_, path)| path.len())
        .filter(|l| *l >= min_node_count)
        .count();

    println!("There are {} rooms in map, the longest path from origin to any path has {} node(s). And there are {} rooms has shortest path which has {} nodes at least", map.len(), node_count_longest_path, node_count_above_min_path_count, min_node_count);
}
