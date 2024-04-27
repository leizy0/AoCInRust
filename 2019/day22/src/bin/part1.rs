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
    let cards_n = 10007;
    let deck = CachedShuffleDeck::new(&techs, cards_n);

    let origin_ind = 2019;
    let target_ind = deck.map_from(origin_ind);
    println!(
        "After shuffle, card(#{}) is at [{}] in deck with {} cards.",
        origin_ind, target_ind, cards_n
    );
}
