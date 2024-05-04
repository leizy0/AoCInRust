use day24::RecursiveArea;

fn main() {
    let input_path = day24::check_args()
        .inspect_err(|e| {
            eprintln!(
                "Failed to get input file path from given arguments, get error({}).",
                e
            )
        })
        .unwrap();
    let area = day24::read_area(&input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read area from given input file({}), get error({}).",
                input_path, e
            )
        })
        .unwrap();

    let target_step_count = 200;
    let mut recur_area = RecursiveArea::new(&area);
    for i in 0..target_step_count {
        println!("Step #{}", i + 1);
        recur_area.step();
    }

    println!(
        "After {} step(s), the given recursive area has {} infested tile(s).",
        target_step_count,
        recur_area.infested_n()
    );
}
