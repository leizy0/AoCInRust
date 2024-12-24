use std::{collections::HashMap, iter};

use anyhow::{Context, Result};
use clap::Parser;
use day21::{CLIArgs, DoorCode, Error, Keypad, Position, UI};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let door_codes = day21::read_door_codes(&args.input_path).with_context(|| {
        format!(
            "Failed to read door codes from given file({}).",
            args.input_path.display()
        )
    })?;

    let middle_level_count = 25;
    let door_ui = Keypad::new_numeric();
    let robot_ui = Keypad::new_directional();
    let ui_chain = iter::repeat_n(&robot_ui as &dyn UI, 1 + middle_level_count)
        .chain(iter::once(&door_ui as &dyn UI))
        .collect::<Vec<&dyn UI>>();
    let mut searched_control_min_keys_n = HashMap::new();
    let mut sum_of_complexities = 0;
    for code in &door_codes {
        let min_keys_n = input_min_keys_n(code, &ui_chain, &mut searched_control_min_keys_n)?;
        sum_of_complexities += min_keys_n * code.number();
    }

    println!(
        "The sum of complexities of given door codes is {}.",
        sum_of_complexities
    );

    Ok(())
}

fn input_min_keys_n(
    code: &DoorCode,
    ui_chain: &[&dyn UI],
    control_records: &mut HashMap<(Position, char, usize), usize>,
) -> Result<usize, Error> {
    let keys = code.text();
    if keys.is_empty() {
        return Ok(0);
    }

    if ui_chain.is_empty() {
        return Err(Error::NoUIAvailable);
    }

    let mut min_keys_n = 0;
    let ui_level = ui_chain.len() - 1;
    let ui = ui_chain[ui_level];
    let mut cur_pos = ui.start_pos();
    for key in keys.chars() {
        min_keys_n += control_min_keys_n_recur(&cur_pos, key, ui_chain, ui_level, control_records)?;
        cur_pos = ui
            .pos(key)
            .expect(&format!("After seeking, given ui should have key {}.", key));
    }

    Ok(min_keys_n)
}

fn control_min_keys_n_recur(
    start_pos: &Position,
    key: char,
    ui_chain: &[&dyn UI],
    target_level: usize,
    control_records: &mut HashMap<(Position, char, usize), usize>,
) -> Result<usize, Error> {
    if target_level == 0 {
        return Ok(1);
    }

    let record_key = (start_pos.clone(), key, target_level);
    if let Some(recorded_min_keys_n) = control_records.get(&record_key) {
        return Ok(*recorded_min_keys_n);
    }

    let target_ui = ui_chain[target_level];
    let (paths, _) = target_ui.seek_key(start_pos, key).expect(&format!(
        "Given ui chain should find the given key({}) from {:?} at level {}.",
        key, start_pos, target_level
    ));
    let mut control_min_keys_n_in_paths = vec![0; paths.len()];
    let control_level = target_level - 1;
    let control_ui = ui_chain[control_level];
    for (path_ind, path) in paths.iter().enumerate() {
        let mut cur_pos = control_ui.start_pos();
        for control_key in path.iter().map(|dir| dir.key()).chain(iter::once('A')) {
            let control_min_keys_n = control_min_keys_n_recur(
                &cur_pos,
                control_key,
                ui_chain,
                control_level,
                control_records,
            )?;
            control_min_keys_n_in_paths[path_ind] += control_min_keys_n;
            cur_pos = control_ui
                .pos(control_key)
                .expect(&format!("The control ui should have key {}.", control_key));
        }
    }

    let control_min_keys_n = control_min_keys_n_in_paths
        .iter()
        .min()
        .copied()
        .expect(&format!(
        "There should be at least one path that can control target ui to seek key {} at level {}.",
        key, control_level
    ));
    control_records
        .entry(record_key)
        .or_insert(control_min_keys_n);

    Ok(control_min_keys_n)
}
