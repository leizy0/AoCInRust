use anyhow::{Context, Result};
use clap::Parser;
use day22::{CLIArgs, CombatRes};

fn main() -> Result<()> {
    let args = CLIArgs::parse();
    let players = day22::read_players(&args.input_path).with_context(|| {
        format!(
            "Failed to read players' information from given input file({}).",
            args.input_path.display()
        )
    })?;

    if players.len() < 2 {
        eprintln!(
            "Expect 2 or more players to start a Combat, given {}.",
            players.len()
        );
    } else {
        const PLAYER_INDS: [usize; 2] = [0, 1];
        let player_ids = [players[PLAYER_INDS[0]].id(), players[PLAYER_INDS[1]].id()];
        println!(
            "Combat(Recursive)! Player {} VS Player {}:",
            player_ids[0], player_ids[1]
        );
        let CombatRes {
            turns_n,
            winner,
            winner_cards,
        } = day22::combat2(&players[PLAYER_INDS[0]], &players[PLAYER_INDS[1]]).with_context(
            || {
                format!(
                    "Failed to simulate combat between player {} and {}.",
                    player_ids[0], player_ids[1]
                )
            },
        )?;
        let winner_cards_n = winner_cards.len();
        let winner_score = winner_cards
            .iter()
            .enumerate()
            .map(|(ind, c)| c * (winner_cards_n - ind))
            .sum::<usize>();
        println!(
            "After {} turn(s), the combat(recursive) is over. Player {} wins, and get score {}.",
            turns_n, player_ids[winner], winner_score
        );
    }

    Ok(())
}
