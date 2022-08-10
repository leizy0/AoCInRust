use day22::{
    map::{self, CaveMap},
    Position,
};

fn main() {
    let input_path = "input.txt";
    let setting = map::load_setting(input_path).expect(&format!(
        "Failed to load setting from input file({})",
        input_path
    ));
    let map = CaveMap::new(&setting);
    let mouse = Position::new(0, 0);
    let risk_level_sum = (mouse.r..=setting.target.r)
        .flat_map(|r| (mouse.c..=setting.target.c).map(move |c| Position::new(r, c)))
        .map(|p| map.at(&p).risk())
        .sum::<usize>();

    println!(
        "Sum of region from mouse {} to target(Included) {} is {}",
        mouse, setting.target, risk_level_sum
    );
}
