use day23::bot;

fn main() {
    let input_path = "input.txt";
    let nanobots = bot::load_bots(input_path).expect(&format!("Failed to load bots from input file({})", input_path));
    let max_rad_bot = nanobots.iter().max_by_key(|b| b.signal_rad()).expect("Can't found nanobot with max signal radius");
    let in_range_count = nanobots.iter().filter(|b| max_rad_bot.is_in_range(b)).count();

    println!("There are {} nanobot(s) in range of the nanobot {} that has the longest signal radium", in_range_count, max_rad_bot);
}
