#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate regex;

use chrono::{Duration, NaiveDate, NaiveDateTime, Timelike};
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

fn main() {
    // Parse input
    let input_path = "input.txt";
    let input_file =
        File::open(input_path).expect(&format!("Failed to open the input file({})", input_path));
    let mut event_list: Vec<Event> = BufReader::new(input_file)
        .lines()
        .map(|l| Event::new(&l.unwrap()).unwrap())
        .collect();

    // Sort and split event by date and time, events between one hour before midniht and one after should be one shift
    event_list.sort_unstable_by_key(|e| e.time());
    let event_date_map = vec_to_map(event_list, |e| {
        let hour = e.time().hour();
        let date = e.time().date();
        match hour {
            23 => date + Duration::days(1),
            0 => date,
            _ => panic!("Invalid event happened in time({})", e.time()),
        }
    });

    // Construct ShiftRecord from events in one day
    let record_list: Vec<ShiftRecord> = event_date_map
        .values()
        .map(|ev| ShiftRecord::new(ev))
        .collect();

    // Map guard id to his(her) ShiftRecord list
    let record_map = vec_to_map(record_list, |sr| sr.guard_id);

    // Find guard with the most minutes asleep
    let max_slp_guard_rec = record_map
        .iter()
        .max_by_key(|(_id, rec_list)| {
            rec_list
                .iter()
                .fold(0, |acc, rec| acc + rec.total_asleep_min())
        })
        .unwrap();

    // Count his(her) asleep times in every minutes of midnight, find minute with largest count
    let (max_asleep_id, max_asleep_rec_list) = max_slp_guard_rec;
    let asleep_list: Vec<&(u32, u32)> = max_asleep_rec_list
        .iter()
        .flat_map(|rec| rec.asleep_range_list())
        .collect();
    let asleep_hist = comp_asleep_hist(&asleep_list);
    let (max_asleep_ind, max_asleep_min) = asleep_hist
        .iter()
        .enumerate()
        .max_by_key(|(_ind, &min)| min)
        .unwrap();
    println!(
        "Guard #{} has the most total asleep minutes({}), the most possible alseep time is at 00:{}",
        max_asleep_id, max_asleep_min, max_asleep_ind
    );

    // Comput and print final result
    let result = max_asleep_id * (max_asleep_ind as u32);
    println!("Final answer is {}", result);
}

fn vec_to_map<E, K, F>(v: Vec<E>, f: F) -> HashMap<K, Vec<E>>
where
    F: Fn(&E) -> K,
    K: std::hash::Hash + std::cmp::Eq,
{
    let mut map = HashMap::new();
    for e in v {
        let key = f(&e);
        let entry = map.entry(key).or_insert(Vec::new());
        entry.push(e);
    }

    map
}

fn comp_asleep_hist(asleep_list: &Vec<&(u32, u32)>) -> Vec<u32> {
    let mut hist = vec![0u32; 60];
    for (beg_min, end_min) in asleep_list {
        let real_end = if *end_min == 0 { 60 } else { *end_min };
        if real_end < *beg_min {
            panic!(
                "Asleep range({}, {}) is invalid, end minute is large than begin minute",
                beg_min, end_min
            );
        }

        for i in (*beg_min)..real_end {
            hist[i as usize] += 1;
        }
    }

    hist
}

#[derive(Copy, Clone, Debug)]
enum Event {
    Shift { time: NaiveDateTime, id: u32 },
    Sleep { time: NaiveDateTime },
    Wake { time: NaiveDateTime },
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
            Some(caps) => {
                return Some(Event::Shift {
                    time: parse_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap(),
                    id: u32::from_str(caps.get(2).unwrap().as_str()).unwrap(),
                })
            }
            _ => (),
        };

        match SLEEP_PATTERN.captures(desc) {
            Some(caps) => {
                return Some(Event::Sleep {
                    time: parse_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap(),
                })
            }
            _ => (),
        };

        match WAKE_PATTERN.captures(desc) {
            Some(caps) => {
                return Some(Event::Wake {
                    time: parse_from_str(caps.get(1).unwrap().as_str(), &DATE_FORMAT).unwrap(),
                })
            }
            _ => (),
        };

        println!("Failed to parse description({})", desc);
        None
    }

    fn time(&self) -> NaiveDateTime {
        match self {
            Event::Shift { time: t, id: _ } => *t,
            Event::Sleep { time: t } => *t,
            Event::Wake { time: t } => *t,
        }
    }
}

struct ShiftRecord {
    date: NaiveDate,
    guard_id: u32,
    asleep_ranges: Vec<(u32, u32)>,
}

impl ShiftRecord {
    fn new(ev: &Vec<Event>) -> ShiftRecord {
        #[derive(PartialEq, Debug)]
        enum ShiftState {
            NotHere,
            Wake,
            Asleep,
        }

        let mut cur_min = 0;
        let mut cur_state = ShiftState::NotHere;
        let mut g_id = 0;
        let mut slp_ranges = Vec::new();
        for e in ev {
            match e {
                Event::Shift { time, id } => {
                    if cur_state != ShiftState::NotHere {
                        panic!("Shift event isn't the first event in one day");
                    }

                    cur_min = time.minute();
                    cur_state = ShiftState::Wake;
                    g_id = *id;
                }
                Event::Wake { time } => {
                    if cur_state != ShiftState::Asleep {
                        panic!("Wake event happen when guard isn't asleep");
                    }

                    let wake_min = time.minute();
                    slp_ranges.push((cur_min, wake_min));
                    cur_state = ShiftState::Wake;
                    cur_min = wake_min;
                }
                Event::Sleep { time } => {
                    if cur_state == ShiftState::NotHere {
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
            asleep_ranges: slp_ranges,
        }
    }

    fn asleep_range_list(&self) -> &Vec<(u32, u32)> {
        &self.asleep_ranges
    }

    fn total_asleep_min(&self) -> u32 {
        self.asleep_ranges
            .iter()
            .fold(0, |acc, (beg_min, end_min)| {
                let this_min = if end_min >= beg_min {
                    end_min - beg_min
                } else {
                    60 - beg_min
                };

                acc + this_min
            })
    }
}
