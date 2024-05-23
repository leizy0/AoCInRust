use assert_cmd::Command;

#[test]
fn part1_output_right_answer() {
    let mut cmd = Command::cargo_bin("part1").unwrap();
    cmd.arg("inputs.txt");

    cmd.assert()
        .success()
        .stdout(predicates::str::contains("439"));
}
