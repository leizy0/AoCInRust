use day22::{self, ShuffleDeck};

fn main() {
    let input_path = day22::check_args().expect("Wrong arguments, no input path found.");
    let techs = day22::read_shuffle(&input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read shuffle techniques from given input({}), get error({}).",
                input_path, e
            )
        })
        .unwrap();

    let cards_n = 119315717514047usize;
    let target_ind = 2020usize;
    let shuffle_count = 101741582076661usize;
    // let cards_n = 10007usize;
    // let target_ind = 3293usize;
    // let shuffle_count = 1usize;
    let deck = ShuffleDeck::new(techs.iter(), cards_n);
    let mut cur_ind = deck.map_from(target_ind);
    let mut map_count = 1usize;
    let cache_interval = 50000000usize;
    while cur_ind != target_ind {
        cur_ind = deck.map_from(cur_ind);
        map_count += 1;
        if map_count % cache_interval == 0 {
            println!("Mapped {} time(s).", map_count);
        }
    }
    println!("After {} time(s), the repeating shuffles form a cycle.", map_count);

    let effective_map_to_count = shuffle_count % map_count;
    for _ in 0..effective_map_to_count {
        cur_ind = deck.map_to(cur_ind);
    }
    println!("After {} times shuffling, the card at [{}] in deck with {} cards is #{}.", shuffle_count, target_ind, cards_n, cur_ind);
}

