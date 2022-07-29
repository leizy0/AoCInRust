use std::collections::{HashMap, HashSet};

use day16::inst::{self, Executor};

fn main() {
    let sample_input_path = "samples.txt";
    let samples = inst::load_samples(sample_input_path)
        .expect(&format!("Failed to load samples from input file({})", sample_input_path));
    let mut code_poss_sets: HashMap<usize, HashSet<usize>> = HashMap::new();
    for sample in samples {
        let guess_insts = inst::guess_insts(&sample);
        if let Some(last_guess) = code_poss_sets.get_mut(&sample.op_code()) {
            *last_guess = last_guess.intersection(&guess_insts).copied().collect();
        } else {
            code_poss_sets.insert(sample.op_code(), guess_insts);
        }
    }

    let op_code_count = code_poss_sets.len();
    let code_map = infer_code_map(code_poss_sets);
    if code_map.len() != op_code_count {
        panic!("Failed to find map to give every operation code a confirmed instruction index");
    }

    let inst_input_path = "instructions.txt";
    let insts = inst::load_insts(inst_input_path).expect(&format!("Failed to load instructions from input file({})", inst_input_path));
    let mut executor = Executor::new();
    for inst in insts {
        match executor.execute_with_map(&inst, &code_map) {
            Err(e) => println!("Failed to execute instruction({}), get error({})", inst, e),
            _ => (),
        }
    }

    println!("After execution, registers = {}", executor.regs());
}

fn infer_code_map(mut code_poss_map: HashMap<usize, HashSet<usize>>) -> HashMap<usize, usize> {
    let op_code_count = code_poss_map.len();
    let mut code_map = HashMap::with_capacity(op_code_count);
    let mut last_confirmed_indices = Vec::new();
    let mut op_code_confirmed_count = 0;
    while op_code_confirmed_count != op_code_count {
        let mut cur_confirmed_indices = Vec::new();
        for (op_code, poss_set) in &mut code_poss_map {
            if poss_set.is_empty() {
                continue;
            }

            for last_confirmed_index in &last_confirmed_indices {
                poss_set.remove(last_confirmed_index);
            }
            if poss_set.len() == 1 {
                let index = poss_set.drain().next().unwrap();
                cur_confirmed_indices.push(index);
                if let Some(last_index) = code_map.insert(*op_code, index) {
                    panic!("Found operation code({}) mapped to different indices(new: {}, old: {})",
                        op_code, index, last_index);
                }
                op_code_confirmed_count += 1;
            }
        }

        if cur_confirmed_indices.is_empty() {
            break;
        } else {
            last_confirmed_indices = cur_confirmed_indices;
        }
    }

    code_map
}