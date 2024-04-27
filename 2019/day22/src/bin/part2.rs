use day22::{self, DeckShuffle};

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
    let mut unit_shuffle = DeckShuffle::new(techs.iter(), cards_n);
    let mut total_shuffle = DeckShuffle::ident(cards_n);
    let mut cur_shuffle_count = shuffle_count;
    while cur_shuffle_count > 0 {
        if cur_shuffle_count & 1 != 0 {
            total_shuffle.append(&unit_shuffle);
        }

        cur_shuffle_count >>= 1;
        unit_shuffle.square();
    }

    let origin_ind = total_shuffle.rev_map(target_ind);
    println!(
        "After {} times shuffling, the card at [{}] in deck with {} cards is #{}.",
        shuffle_count, target_ind, cards_n, origin_ind
    );
}
