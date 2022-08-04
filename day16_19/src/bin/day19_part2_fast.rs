use std::collections::HashSet;

fn main() {
    const INPUT_NUMBER:usize = 10551276; // this number is from intermediate state(General register #5) of part 2 execution.
    let mut divisors = HashSet::new();
    for i in 1..((INPUT_NUMBER as f64).sqrt().ceil() as usize) {
        if INPUT_NUMBER % i == 0 {
            divisors.insert(i);
            divisors.insert(INPUT_NUMBER / i);
        }
    }

    println!("{} has {} divisors, sum is {}", INPUT_NUMBER, divisors.len(), divisors.iter().sum::<usize>());
}