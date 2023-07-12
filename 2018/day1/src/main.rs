use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() {
    let input_path = "./input.txt";
    let mut freq = 0i32;
    let mut freq_set = HashSet::new();

    let file = File::open(input_path).expect(&format!("Failed to open input file({})", input_path));

    // Parse the change list
    let chg_list: Vec<i32> = BufReader::new(file)
        .lines()
        .map(|l| {
            let s = l.unwrap();
            i32::from_str(s.as_str())
                .expect(&format!("Failed to parse line({})", &s))
        })
        .collect();

    let chg_count = chg_list.len();
    let mut ind = 0;

    // Loop change list, record frequencies encountered, find the first repeat frequency
    loop {
        freq_set.insert(freq);
        freq += chg_list[ind % chg_count];
        if freq_set.contains(&freq) {
            println!("Repeat frequency is {}, after {} changes", freq, ind);
            break;
        }

        ind += 1;
    }
}
