use std::env::args;

use day22::{self, Deck, Error};

fn main() {
    let input_path = check_args().expect("Wrong arguments, no input path found.");
    let techs = day22::read_shuffle(&input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read shuffle techniques from given input({}), get error({}).",
                input_path, e
            )
        })
        .unwrap();
    let deck_n = 10007;
    let mut deck = Deck::new(deck_n);
    for tech in &techs {
        deck.shuffle(tech);
    }

    let target_n = 2019;
    if let Some(target_ind) = deck.find(target_n) {
        println!(
            "After shuffle, card({}) is at #{} in deck.",
            target_n, target_ind
        );
    } else {
        eprintln!("The card({}) isn't in deck.", target_n);
    }
}

fn check_args() -> Result<String, Error> {
    let args_n = args().len();
    let expect_n = 2;
    if args_n != expect_n {
        eprintln!("Wrong number of arguments, expect one.");
        println!("Usage: {} INPUT_FILE_PATH", args().next().unwrap());
        Err(Error::WrongNumberOfArgs(args_n, expect_n))
    } else {
        Ok(args().skip(1).next().unwrap().to_string())
    }
}
