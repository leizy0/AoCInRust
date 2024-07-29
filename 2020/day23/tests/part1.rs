use assert_cmd::Command;
use predicates::prelude::predicate::str;

#[test]
fn part1_output_right_answer() {
    let mut cmd = Command::cargo_bin("part1").unwrap();
    cmd.arg("215694783");

    cmd.assert().success().stdout(str::contains("46978532"));
}
