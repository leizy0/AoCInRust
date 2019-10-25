#[macro_use] extern crate lazy_static;
extern crate chrono;
extern crate regex;

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::collections::HashMap;
use chrono::{NaiveDate, NaiveDateTime, Timelike};
use regex::Regex;


fn main() {
    // Parse input
    let input_path = "input.txt";
    let input_file = File::open(input_path).expect(&format!("Failed to open the input file({})", input_path));
    let mut event_list: Vec<Event> = BufReader::new(input_file)
        .lines()
        .map(|l| Event::new(&l.unwrap()).unwrap())
        .collect();

    // Sort and split event by date
    event_list.sort_unstable_by_key(|e| e.time());
    let event_date_map = vec_to_map(event_list, |e| e.time().date());
    
    // Construct ShiftRecord from events in one day
    let record_list: Vec<ShiftRecord> = event_date_map.values().map(|ev| ShiftRecord::new(ev)).collect();

    // Map guard id to his(her) ShiftRecord list
    let record_map = vec_to_map(record_list, |sr| sr.guard_id);

    // Find guard with the most minutes asleep

    // Count his(her) asleep times in every minutes of midnight, find minute with largest count

    // Comput and print final result
}

fn vec_to_map<E, K, F>(v: Vec<E>, f: F) -> HashMap<K, Vec<E>> 
where F: Fn(&E) -> K, K: std::hash::Hash + std::cmp::Eq {
    let mut map = HashMap::new();
    for e in v {
        let key = f(&e);
        let entry = map.entry(key).or_insert(Vec::new());
        entry.push(e);
    }

    map
}

#[derive(Copy, Clone)]
enum Event {
    Shift{ time: NaiveDateTime, id: u32 },
    Sleep{ time: NaiveDateTime },
    Wake{ time: NaiveDateTime }
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

        let parse_from_str = NaiveDateTime::parse_from_str;
        match SHIFT_PATTERN.captures(desc) {
            Some(caps) => return Some(Event::Shift{ time: parse_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap(), id: u32::from_str(caps.get(2).unwrap().as_str()).unwrap()}),
            _ => ()
        };

        match SLEEP_PATTERN.captures(desc) {
            Some(caps) => return Some(Event::Sleep{ time: parse_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap()}),
            _ => ()
        };

        match WAKE_PATTERN.captures(desc) {
            Some(caps) => return Some(Event::Wake{ time: parse_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap()}),
            _ => ()
        };

        println!("Failed to parse description({})", desc);
        None
    }

    fn time(&self) -> NaiveDateTime {
        match self {
            Event::Shift{time: t, id: _} => *t,
            Event::Sleep{time: t} => *t,
            Event::Wake{time: t} => *t
        }
    }
}

struct ShiftRecord {
    date: NaiveDate,
    guard_id: u32,
    asleep_ranges: Vec<(u32, u32)>
}

impl ShiftRecord {
    fn new(ev: &Vec<Event>) -> ShiftRecord {
        #[derive(PartialEq)]
        enum ShiftState {
            Gone,
            Wake,
            Asleep
        }

        let mut cur_min = 0;
        let mut cur_state = ShiftState::Gone;
        let mut g_id = 0;
        let mut slp_ranges = Vec::new();
        for e in ev {
            match e {
                Event::Shift{time, id} => {
                    if cur_state != ShiftState::Gone {
                        panic!("Shift event isn't the first event in one day");
                    }

                    cur_min = time.minute();
                    cur_state = ShiftState::Wake;
                    g_id = *id;
                },
                Event::Wake{time} => {
                    if cur_state != ShiftState::Asleep {
                        panic!("Wake event happen when guard isn't asleep");
                    }

                    let wake_min = time.minute();
                    slp_ranges.push((cur_min, wake_min));
                    cur_state = ShiftState::Wake;
                    cur_min = wake_min;
                },
                Event::Sleep{time} => {
                    if cur_state == ShiftState::Gone {
                        panic!("Can't sleep when guard isn't here");
                    }

                    cur_min = time.minute();
                    cur_state = ShiftState::Asleep;
                }
            }
        }

        if cur_state == ShiftState::Asleep {
            slp_ranges.push((cur_min, 0));
        }

        ShiftRecord {
            date: ev[0].time().date(),
            guard_id: g_id,
            asleep_ranges: slp_ranges
        }
    }
}