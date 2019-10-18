#[macro_use] extern crate lazy_static;
extern crate chrono;
extern crate regex;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use chrono::{DateTime, Utc, TimeZone};
use regex::Regex;

fn main() {
    // Parse input
    let input_path = "input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open the input file({})", input_path));
    let event_list: Vec<Event> = BufReader::new(input_file)
        .lines()
        .map(|l| Event::new(&l.unwrap()).unwrap())
        .collect();

    // Sort and split event by date
    
    // Construct DateEvent from events in one day

    // Map guard id to his(her) DateEvent

    // Find guard with the most minutes asleep

    // Count his(her) asleep times in every minutes of midnight, find minute with largest count

    // Comput and print final result
}

enum Event {
    Shift{ time: DateTime<Utc>, id: u32 },
    Sleep{ time: DateTime<Utc> },
    Wake{ time: DateTime<Utc> }
}

impl Event {
    fn new(desc: &str) -> Option<Event> {
        lazy_static! {
            static ref DATE_FORMAT: String = "%Y-%m-%d %H:%M".to_string();
            //[1518-09-18 23:57] Guard #2273 begins shift
            static ref SHIFT_PATTERN: Regex = Regex::new(r"\[(\d+-\d+-\d+ \d+:\d+)\] Guard #(\d+) begins shift").unwrap();
            //[1518-06-23 00:05] falls asleep
            static ref SLEEP_PATTERN: Regex = Regex::new(r"\[(\d+-\d+-\d+ \d+:\d+)\] falls asleep").unwrap();
            //[1518-04-26 00:51] wakes up
            static ref WAKE_PATTERN: Regex = Regex::new(r"\[(\d+-\d+-\d+ \d+:\d+)\] wakes up").unwrap();
        }

        match SHIFT_PATTERN.captures(desc) {
            Some(caps) => return Some(Event::Shift{ time: Utc.datetime_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap(), id: u32::from_str(caps.get(2).unwrap().as_str()).unwrap()}),
            _ => ()
        };

        match SLEEP_PATTERN.captures(desc) {
            Some(caps) => return Some(Event::Sleep{ time: Utc.datetime_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap()}),
            _ => ()
        };

        match WAKE_PATTERN.captures(desc) {
            Some(caps) => return Some(Event::Wake{ time: Utc.datetime_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap()}),
            _ => ()
        };

        println!("Failed to parse description({})", desc);
        None
    }
}