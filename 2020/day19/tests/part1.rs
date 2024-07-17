use assert_cmd::Command;
use predicates::prelude::predicate::str;

#[test]
fn part1_output_right_answer() {
    let mut cmd = Command::cargo_bin("part1_2").unwrap();
    cmd.arg("part1_inputs.txt");

    cmd.assert().success().stdout(str::contains("208"));
}
