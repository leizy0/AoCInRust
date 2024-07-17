use assert_cmd::Command;
use predicates::prelude::predicate::str;

#[test]
fn part2_output_right_answer() {
    let mut cmd = Command::cargo_bin("part1_2").unwrap();
    cmd.arg("part2_inputs.txt");

    cmd.assert().success().stdout(str::contains("316"));
}
