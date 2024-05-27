use assert_cmd::Command;
use predicates::{boolean::PredicateBooleanExt, prelude::predicate::str, BoxPredicate};

#[test]
fn part2_output_right_answer() {
    let mut cmd = Command::cargo_bin("part2").unwrap();
    cmd.arg("inputs.txt");

    let expect_numbers = [81, 292, 89, 101, 44];
    let pred = expect_numbers
        .iter()
        .map(|n| BoxPredicate::new(str::contains(n.to_string())))
        .reduce(|total_p, this_p| BoxPredicate::new(total_p.and(this_p)))
        .unwrap();
    cmd.assert().success().stdout(pred);
}
