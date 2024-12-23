use std::{collections::HashMap, iter};

use anyhow::{Context, Result};
use clap::Parser;
use day21::{CLIArgs, Error, Keypad, Position, Robot, UI};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let door_codes = day21::read_door_codes(&args.input_path).with_context(|| {
        format!(
            "Failed to read door codes from given file({}).",
            args.input_path.display()
        )
    })?;

    let middle_level_count = 25;
    let mut searched_key_seeking_min_keys_n = HashMap::new();
    let mut sum_of_complexities = 0;
    for code in &door_codes {
        let first_robot = Robot::new(Keypad::new_numeric());
        let first_robot_key_seqs = first_robot.input(code.text())?;
        let final_robot_min_keys_n = first_robot_key_seqs
            .iter()
            .map(|robot_key_seq| {
                robot_min_keys_n(
                    robot_key_seq,
                    middle_level_count,
                    &mut searched_key_seeking_min_keys_n,
                )
                .unwrap()
            })
            .min()
            .unwrap();
        sum_of_complexities += final_robot_min_keys_n * code.number();
    }

    println!(
        "The sum of complexities of given door codes is {}.",
        sum_of_complexities
    );

    Ok(())
}

fn robot_min_keys_n(
    robot_keys: &[char],
    robot_level: usize,
    searched_key_seeking_min_keys_n: &mut HashMap<(Position, char, usize), usize>,
) -> Result<usize, Error> {
    let mut min_keys_n = 0;
    let robot_keypad = Keypad::new_directional();
    let mut cur_position = robot_keypad.start_pos();
    for key in robot_keys {
        let seek_min_keys_n = seek_key_min_keys_n_recur(
            &cur_position,
            *key,
            robot_level,
            searched_key_seeking_min_keys_n,
        )?;
        min_keys_n += seek_min_keys_n;
        cur_position = robot_keypad
            .pos(*key)
            .expect(&format!("The robot keypad ui should have key {}.", *key));
    }

    Ok(min_keys_n)
}

fn seek_key_min_keys_n_recur(
    start_pos: &Position,
    key: char,
    level: usize,
    searched_key_seeking_min_keys_n: &mut HashMap<(Position, char, usize), usize>,
) -> Result<usize, Error> {
    let keypad = Keypad::new_directional();
    if level == 0 {
        return Ok(1);
    }

    let searched_map_key = (start_pos.clone(), key, level);
    if let Some(min_keys_n) = searched_key_seeking_min_keys_n.get(&searched_map_key) {
        return Ok(*min_keys_n);
    }

    let (all_path, _) = keypad.seek_key(start_pos, key).expect(&format!(
        "Robot should find the given key({}) from {:?} at level {}.",
        key, start_pos, level
    ));
    let mut all_min_keys_n = vec![0; all_path.len()];
    for (path_ind, dirs) in all_path.iter().enumerate() {
        let mut cur_pos = keypad.start_pos();
        for low_level_key in dirs.iter().map(|dir| dir.key()).chain(iter::once('A')) {
            let low_level_min_keys_n = seek_key_min_keys_n_recur(
                &cur_pos,
                low_level_key,
                level - 1,
                searched_key_seeking_min_keys_n,
            )?;
            all_min_keys_n[path_ind] += low_level_min_keys_n;
            cur_pos = keypad.pos(low_level_key).expect(&format!(
                "The robot keypad ui should have key {}.",
                low_level_key
            ));
        }
    }

    let min_keys_n = all_min_keys_n.iter().min().copied().expect(&format!(
        "There should be at least one path that can seek key {} at level {}.",
        key, level
    ));
    searched_key_seeking_min_keys_n
        .entry(searched_map_key)
        .or_insert(min_keys_n);

    Ok(min_keys_n)
}
