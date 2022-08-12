use std::{
    collections::{HashMap, HashSet},
};

use day23::bot::{self, Nanobot, Position};

fn main() {
    let input_path = "input.txt";
    let nanobots = bot::load_bots(input_path).expect(&format!(
        "Failed to load bots from input file({})",
        input_path
    ));
    // Find nanobot set has the most nanobots in which any nanobot has intersection with each other
    let intersect_mat = comp_intersect_mat(&nanobots[..]);
    let max_nodes_set = comp_max_2_2_intersected_set(&intersect_mat);
    assert!(is_all_mutually_neighbor(&max_nodes_set, &intersect_mat));

    // Binary search the minimum radius with which nanobot center in origin can be in range of all nanobots in set computed above.
    let max_rad_bot = nanobots
        .iter()
        .max_by_key(|b| b.signal_rad())
        .expect("Can't found nanobot with max signal radius");
    let origin = Position::new(0, 0, 0);
    let mut min_dist = 0;
    let mut max_dist = origin.mht_dist(&max_rad_bot.pos()) + max_rad_bot.signal_rad();
    while min_dist < max_dist {
        let cur_dist = (min_dist + max_dist) / 2;
        println!(
            "Try distance {} in range({}, {})",
            cur_dist, min_dist, max_dist
        );
        let virt_bot = Nanobot::new(&origin, cur_dist);
        if max_nodes_set
            .iter()
            .all(|ind| virt_bot.has_intersection(&nanobots[*ind]))
        {
            max_dist = cur_dist - 1;
        } else {
            min_dist = cur_dist + 1;
        }
    }

    println!("The shortest manhattan distance between any position that is in range of the most nanobots is {}", min_dist);
}

fn comp_intersect_mat(bots: &[Nanobot]) -> Vec<HashSet<usize>> {
    let bot_count = bots.len();

    (0..bot_count)
        .map(|i| {
            (0..bot_count)
                .filter(|j| *j != i && bots[i].has_intersection(&bots[*j]))
                .collect::<HashSet<_>>()
        })
        .collect::<Vec<_>>()
}

// Compute two two directly connected set with maximum node count
fn comp_max_2_2_intersected_set(neighbor_mat: &Vec<HashSet<usize>>) -> Vec<usize> {
    let mut min_count = 0;
    let mut max_count = neighbor_mat.len();
    while min_count <= max_count {
        let mut init_mat = neighbor_mat
            .iter()
            .enumerate()
            .map(|(ind, l)| (ind, l.clone()))
            .collect::<HashMap<_, _>>();

        let cur_count = (min_count + max_count) / 2;
        println!("Try node count {} in range({}, {})", cur_count, min_count, max_count);
        loop {
            let del_ind = if let Some((ind, l)) = init_mat.iter().min_by_key(|&(_, l)| l.len()) {
                if l.len() >= cur_count - 1 {
                    break;
                } else {
                    *ind
                }
            } else {
                break;
            };

            init_mat.remove(&del_ind)
                .unwrap()
                .iter()
                .for_each(|ind| {
                    assert!(init_mat[ind].contains(&del_ind));
                    init_mat.get_mut(ind).unwrap().remove(&del_ind);
                });
        }

        if init_mat.len() > cur_count {
            min_count = cur_count + 1;
        } else if init_mat.len() == cur_count {
            return init_mat.keys().copied().collect::<Vec<_>>();
        } else {
            max_count = cur_count - 1;
        }
    }

    panic!("Unreachable failure");
}

fn is_all_mutually_neighbor(node_ind_set: &[usize], neighbor_mat: &Vec<HashSet<usize>>) -> bool {
    node_ind_set
        .iter()
        .flat_map(|l_ind| node_ind_set
            .iter()
            .filter(move |ind| *ind != l_ind)
            .map(move |r_ind| (*l_ind, *r_ind)))
        .all(|(l_ind, r_ind)| neighbor_mat[l_ind].contains(&r_ind))
}
