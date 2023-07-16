use day3::wire::read_wires;

fn main() {
    let wires_file = "inputs.txt";
    let wires =
        read_wires(wires_file).expect(&format!("Failed to read wires from file({})", wires_file));
    assert!(wires.len() == 2);
    let min_mht_cross_point = wires[0]
        .cross(&wires[1])
        .into_iter()
        .min_by_key(|p| p.total_len())
        .expect("Two given wires don't have any cross point");
    println!(
        "Two given wire have an cross point({:?}) which has the least total length({})",
        min_mht_cross_point,
        min_mht_cross_point.total_len()
    );
}
