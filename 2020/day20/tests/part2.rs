use assert_cmd::Command;
use predicates::prelude::predicate::str;

#[test]
fn part2_output_right_answer() {
    let mut cmd = Command::cargo_bin("part2").unwrap();
    cmd.args(["inputs_tiles.txt", "inputs_mask.txt"]);

    cmd.assert().success().stdout(str::contains("2442"));
}
