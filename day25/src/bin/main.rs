use std::collections::{HashSet, LinkedList};

use day25::point::{self, Point};

fn main() {
    let input_path = "input.txt";
    let points = point::load_points(input_path).expect(&format!(
        "Failed to load points from input file({})",
        input_path
    ));
    let neighbor_mat = comp_near_neighbor_mat(&points);
    let components = comp_components(&neighbor_mat);
    println!(
        "There are {} constellation(s) in given {} point(s).",
        components.len(),
        points.len()
    );
}

fn comp_near_neighbor_mat(points: &Vec<Point>) -> Vec<Vec<usize>> {
    let point_count = points.len();
    (0..point_count)
        .map(|l_ind| {
            (0..point_count)
                .filter(|r_ind| *r_ind != l_ind && points[l_ind].mht_dist(&points[*r_ind]) <= 3)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

fn comp_components(neighbor_mat: &Vec<Vec<usize>>) -> Vec<HashSet<usize>> {
    let point_count = neighbor_mat.len();
    let mut visited_marks = vec![false; point_count];
    let mut components = Vec::new();
    while let Some(start_ind) = visited_marks.iter().position(|m| !*m) {
        let mut scan_queue = LinkedList::from([start_ind]);
        let mut component = HashSet::new();
        while let Some(cur_ind) = scan_queue.pop_front() {
            if !visited_marks[cur_ind] {
                component.insert(cur_ind);
                visited_marks[cur_ind] = true;
            }

            scan_queue.extend(
                neighbor_mat[cur_ind]
                    .iter()
                    .filter(|ind| !visited_marks[**ind]),
            );
        }
        components.push(component);
    }

    components
}
