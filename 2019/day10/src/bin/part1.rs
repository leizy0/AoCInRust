use day10::asteroid::read_map;

fn main() {
    let input_path = "inputs.txt";
    let asteroid_map = match read_map(input_path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "Failed to read asteroid map from given input file({}), get error({})",
                input_path, e
            );
            return;
        }
    };

    let asteroid_count = asteroid_map.asteroid_count();
    match (0..asteroid_count)
        .map(|ind| (ind, asteroid_map.detect_count(ind).unwrap()))
        .max_by_key(|(_, c)| *c)
    {
        Some((ind, c)) => println!(
            "Asteroid({:?}) in map has maximum detect count({})",
            asteroid_map.asteroid(ind).unwrap(),
            c
        ),
        None => eprintln!("There's no asteroid in given map."),
    }
}
