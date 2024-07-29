use assert_cmd::Command;
use predicates::prelude::predicate::str;

#[test]
fn part2_output_right_answer() {
    let mut cmd = Command::cargo_bin("part2").unwrap();
    cmd.arg("215694783");

    cmd.assert().success().stdout(str::contains("163035127721"));
}
