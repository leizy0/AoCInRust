use day6::orbit::{read_orbits, Object};

fn main() {
    let input_file = "inputs.txt";
    let orbit_tree = read_orbits(input_file).expect(&format!(
        "Failed to read orbit tree from file({})",
        input_file
    ));

    let you = Object::from("YOU");
    let santa = Object::from("SAN");
    if !orbit_tree.has_obj(&you) || !orbit_tree.has_obj(&santa) {
        eprintln!(
            "Target objects({} and {}) not found in given orbits",
            you, santa
        );
        return;
    }

    let mut you_path = orbit_tree.orbit_path_of(&you).unwrap();
    let mut santa_path = orbit_tree.orbit_path_of(&santa).unwrap();
    you_path.reverse();
    santa_path.reverse();

    // assert!(you_path[0] == santa_path[0]);
    let mut closest_common_obj_ind = -1i32;
    for obj_ind in 0..(you_path.len().min(santa_path.len())) {
        if you_path[obj_ind] != santa_path[obj_ind] {
            break;
        } else {
            closest_common_obj_ind = obj_ind as i32;
        }
    }

    if closest_common_obj_ind < 0 {
        eprintln!(
            "Orbit paths of {} and {} have no common object, they can never reach each other.",
            you, santa
        );
    } else {
        let you_dist = (you_path.len() as i32 - 1) - 1 - closest_common_obj_ind;
        let santa_dist = (santa_path.len() as i32 - 1) - 1 - closest_common_obj_ind;
        println!(
            "The closest common object in orbit paths of {} and {} is {}(@ position {} from common root), so they need {} jumps to reach each other.",
            you, santa, orbit_tree.obj(you_path[closest_common_obj_ind as usize]).unwrap(), closest_common_obj_ind, you_dist + santa_dist
        );
    }
}
