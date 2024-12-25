use std::collections::{BTreeSet, HashSet};

use anyhow::{Context, Result};
use clap::Parser;
use day24::{CLIArgs, Circuit};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let (_, mut circuit) = day24::read_circuit_info(&args.input_path).with_context(|| {
        format!(
            "Failed to read circuit information in given file({}).",
            args.input_path.display()
        )
    })?;

    let max_swap_count = 4;
    let function = |x: usize, y: usize| x + y;
    if let Some(swapped_indices) =
        swap_indices_to_function_correct(&mut circuit, function, max_swap_count)
    {
        let mut final_swapped_wire_names = swapped_indices
            .iter()
            .map(|ind| circuit.gate(*ind).unwrap().output_wire().name().to_string())
            .collect::<Vec<_>>();
        final_swapped_wire_names.sort_unstable();
        println!(
            "The circuit can get right sum answer after swapped wires: {}.",
            final_swapped_wire_names.join(",")
        );
    } else {
        eprintln!("There's no way to correct the output of given circuit with at most {} swaps of gate output wires.", max_swap_count);
    }

    Ok(())
}

fn swap_indices_to_function_correct(
    circuit: &mut Circuit,
    function: fn(usize, usize) -> usize,
    max_swap_count: usize,
) -> Option<HashSet<usize>> {
    assert!(circuit.x_bits_n() == circuit.y_bits_n());
    let mut failed_swap_pairs = HashSet::new();
    swap_indices_to_function_correct_per_bit_recur(
        circuit,
        function,
        &HashSet::new(),
        &BTreeSet::new(),
        &mut failed_swap_pairs,
        max_swap_count,
        0,
    )
}

fn swap_indices_to_function_correct_per_bit_recur(
    circuit: &mut Circuit,
    function: fn(usize, usize) -> usize,
    swapped_indices: &HashSet<usize>,
    swapped_pairs: &BTreeSet<(usize, usize)>,
    failed_swap_pairs: &mut HashSet<BTreeSet<(usize, usize)>>,
    left_swap_count: usize,
    bit_ind: usize,
) -> Option<HashSet<usize>> {
    if bit_ind >= circuit.x_bits_n() {
        return Some(swapped_indices.clone());
    }

    if failed_swap_pairs.contains(swapped_pairs) {
        return None;
    }

    if let Some(test_output_bit_marks) = test_circuit_at_bit(circuit, function, bit_ind) {
        // Test failed, swap gate
        let mut swap_gate_candidate_indices = HashSet::new();
        for output_bit_ind in (0..circuit.z_bits_n())
            .filter(|output_bit_ind| (1 << output_bit_ind) & test_output_bit_marks != 0)
        {
            swap_gate_candidate_indices
                .extend(circuit.gate_indices_output_to_bit(output_bit_ind).iter());
        }

        if left_swap_count == 0 {
            failed_swap_pairs.insert(swapped_pairs.clone());
            return None;
        }

        let swap_gate_candidate_indices =
            Vec::from_iter(swap_gate_candidate_indices.iter().copied());
        let swap_candidate_n = swap_gate_candidate_indices.len();
        for i in 0..swap_candidate_n {
            let left_gate_ind = swap_gate_candidate_indices[i];
            if swapped_indices.contains(&left_gate_ind) {
                continue;
            }
            for j in (i + 1)..swap_candidate_n {
                let right_gate_ind = swap_gate_candidate_indices[j];
                if swapped_indices.contains(&right_gate_ind) {
                    continue;
                }

                let mut new_swapped_pairs = swapped_pairs.clone();
                new_swapped_pairs.insert((
                    left_gate_ind.min(right_gate_ind),
                    left_gate_ind.max(right_gate_ind),
                ));
                if failed_swap_pairs.contains(&new_swapped_pairs) {
                    continue;
                }

                circuit.swap_gate_pair_output(left_gate_ind, right_gate_ind);
                if (0..=bit_ind).all(|test_bit_ind| {
                    test_circuit_at_bit(circuit, function, test_bit_ind).is_none()
                }) {
                    let mut new_swapped_indices = swapped_indices.clone();
                    new_swapped_indices.insert(left_gate_ind);
                    new_swapped_indices.insert(right_gate_ind);
                    if let Some(final_swapped_indices) =
                        swap_indices_to_function_correct_per_bit_recur(
                            circuit,
                            function,
                            &new_swapped_indices,
                            &new_swapped_pairs,
                            failed_swap_pairs,
                            left_swap_count - 1,
                            bit_ind + 1,
                        )
                    {
                        return Some(final_swapped_indices);
                    }
                }

                // Failed, swap back.
                failed_swap_pairs.insert(new_swapped_pairs);
                circuit.swap_gate_pair_output(left_gate_ind, right_gate_ind);
            }
        }
    } else {
        // Test succeed, continue recursion.
        return swap_indices_to_function_correct_per_bit_recur(
            circuit,
            function,
            swapped_indices,
            swapped_pairs,
            failed_swap_pairs,
            left_swap_count,
            bit_ind + 1,
        );
    }

    failed_swap_pairs.insert(swapped_pairs.clone());
    None
}

fn test_circuit_at_bit(
    circuit: &mut Circuit,
    function: fn(usize, usize) -> usize,
    bit_ind: usize,
) -> Option<usize> {
    let test_values = [0, 1 << bit_ind];
    let mut output_bit_marks = 0;
    let mut test_succeed = true;
    for (x, y) in test_values
        .iter()
        .flat_map(|x| test_values.iter().map(move |y| (x, y)))
    {
        if let Some(output) = circuit.simulate_with_xy(*x, *y) {
            let expect_output = function(*x, *y);
            if output != expect_output {
                test_succeed = false;
            }

            output_bit_marks |= output ^ expect_output;
        } else {
            test_succeed = false;
        }
    }

    if test_succeed {
        None
    } else {
        Some(output_bit_marks)
    }
}
