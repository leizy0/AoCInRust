use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn part2_output_right_answer() {
    let mut cmd = Command::cargo_bin("part2").unwrap();
    cmd.arg("inputs.txt");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("42275090"));
}
