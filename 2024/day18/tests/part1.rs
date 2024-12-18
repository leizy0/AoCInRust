use assert_cmd::Command;
use predicates::prelude::predicate::str;

#[test]
fn part1_output_right_answer() {
    let mut cmd = Command::cargo_bin("part1").unwrap();
    cmd.arg("inputs.txt").arg("71").arg("1024");

    cmd.assert().success().stdout(str::contains("270"));
}
