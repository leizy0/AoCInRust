use day22::{
    map::{self, CaveMap},
    play::{self, Player, Tool},
    Position,
};

fn main() {
    let input_path = "input.txt";
    let setting = map::load_setting(input_path).expect(&format!(
        "Failed to load setting from input file({})",
        input_path
    ));
    let map = CaveMap::new(&setting);
    let init_player = Player::new(&Position::new(0, 0), Tool::Torch);
    let end_player = Player::new(&setting.target, Tool::Torch);
    let actions = play::fastest_plan_to(&init_player, &map, &end_player).expect(&format!(
        "Failed to find the fastest plan from {} to {}",
        init_player, end_player
    ));
    let cost_mins = actions.iter().map(|a| a.cost()).sum::<usize>();

    println!(
        "The fastest plan from {} to {} costs {} minutes",
        init_player, end_player, cost_mins
    );
}
