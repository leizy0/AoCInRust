use day22::{self, CachedShuffleDeck};

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
    // let cards_n = 10007usize;
    let deck = CachedShuffleDeck::new(techs.iter(), cards_n);
    let target_ind = 2020usize;
    // let target_ind = 3293usize;
    let shuffle_count = 101741582076661usize;
    // let shuffle_count = 1usize;
    let mut shuffle_map_cache = vec![target_ind];
    let mut cur_ind = deck.map_to(target_ind);
    let mut map_count = 1usize;
    let cache_interval = 5000000usize;
    while cur_ind != target_ind && map_count < shuffle_count {
        cur_ind = deck.map_to(cur_ind);
        map_count += 1;
        if map_count % cache_interval == 0 {
            shuffle_map_cache.push(cur_ind);
            println!("Mapped {} time(s).", map_count);
        }
    }
    println!("There are {} elements in the map cache.", shuffle_map_cache.len());

    let target_card_ind = if map_count < shuffle_count {
        let effective_shuffle_count = shuffle_count % map_count;
        let cache_ind = effective_shuffle_count / cache_interval;
        cur_ind = shuffle_map_cache[cache_ind];
        let left_map_count = effective_shuffle_count % cache_interval;
        for _ in 0..left_map_count {
            cur_ind = deck.map_to(cur_ind);
        }

        cur_ind
    } else {
        cur_ind
    };

    println!("After {} times shuffling, the card at [{}] in deck with {} cards is #{}.", shuffle_count, target_ind, cards_n, target_card_ind);
}

