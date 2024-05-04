use std::collections::HashSet;

fn main() {
    let input_path = day24::check_args()
        .inspect_err(|e| {
            eprintln!(
                "Failed to get input file path from given arguments, get error({}).",
                e
            )
        })
        .unwrap();
    let mut area = day24::read_area(&input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read area from given input file({}), get error({}).",
                input_path, e
            )
        })
        .unwrap();

    let mut appeared_bio_ratings = HashSet::new();
    let mut step_count = 0usize;
    loop {
        let cur_bio_rating = area.bio_rating();
        if !appeared_bio_ratings.insert(cur_bio_rating) {
            println!(
                "After {} steps, biodiversity rating({}) of given area repeated.",
                step_count, cur_bio_rating
            );
            break;
        }

        area.step();
        step_count += 1;
        println!("After #{} step(s):\n{}", step_count, area);
    }
}
