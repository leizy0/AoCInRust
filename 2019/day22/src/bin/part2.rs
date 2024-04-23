use std::collections::HashSet;

use day22::{self, Deck};

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
    let deck = Deck::new(cards_n);
    let target_ind = 2020usize;
    // let target_ind = 3293usize;
    let mut shuffle_map_link = vec![target_ind];
    let mut cur_ind = deck.shuffle_map_to(&techs, target_ind);
    while cur_ind != target_ind {
        shuffle_map_link.push(cur_ind);
        cur_ind = deck.shuffle_map_to(&techs, cur_ind);
    }

    let shuffle_count = 101741582076661usize;
    // let shuffle_count = 1usize;
    let effective_shuffle_count = shuffle_count % shuffle_map_link.len();
    let target_card_ind = shuffle_map_link[effective_shuffle_count];
    println!("After {} times shuffling, the card at [{}] in deck with {} cards is #{}.", shuffle_count, target_ind, cards_n, target_card_ind);
}

