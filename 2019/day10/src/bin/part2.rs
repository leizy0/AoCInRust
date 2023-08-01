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
    (0..asteroid_count)
        .map(|ind| (ind, asteroid_map.detect_count(ind).unwrap()))
        .max_by_key(|(_, c)| *c)
        .map(|(ind, _)| {
            asteroid_map.vaporize_order(ind).map(|ind_v| {
                for (i, &v_a_ind) in ind_v.iter().enumerate() {
                    println!(
                        "{}th asteroid is {:?} which has been vaporized by {:?}",
                        i + 1,
                        asteroid_map.asteroid(v_a_ind).unwrap(),
                        asteroid_map.asteroid(ind).unwrap()
                    )
                }
            })
        });
}
