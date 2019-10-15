use std::fs::File;
use std::io::{Write, BufRead, BufReader, ErrorKind};
use std::str::FromStr;

fn main() {
    let input_path = "./input.txt";
    let mut freq = 0i32;

    let file = File::open(input_path).expect(&format!("Failed to open input file({})", input_path));
    for line_res in BufReader::new(file).lines() {
        let change = match line_res {
            Ok(s) => i32::from_str(s.as_str()).expect(&format!("Failed to parse line({})", s)),
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                writeln!(std::io::stderr(), "Failed to process line, get error({})", e).unwrap();
                std::process::exit(1)
            }
        };

        freq += change;
    }

    println!("The final frequency is {}", freq);
}
