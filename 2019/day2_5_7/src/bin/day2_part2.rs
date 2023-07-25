use day2_5_7::int_code::{com::IntCodeComputer, read_int_code};

fn main() {
    let int_code_file = "day2_inputs.txt";
    let int_code = read_int_code(int_code_file)
        .expect(&format!("Failed to read int code from {}", int_code_file));

    let mut computer = IntCodeComputer::new(false);
    for code1 in 0..3 {
        for code2 in 0..=9 {
            let mut int_code_image = int_code.clone();
            int_code_image[1] = code1;
            int_code_image[2] = code2;
            match computer.execute(int_code_image, Vec::new()) {
                Ok(res) => println!(
                    "{:>2} | {:>2}: After {} steps, program halts, code[0] = {}",
                    code1,
                    code2,
                    res.step_count(),
                    res.image()[0]
                ),
                Err(e) => println!("Failed to run int code, get error({})", e),
            }
        }
    }

    // Using linear regression, after program halts, code[0] = code[1] * 300000 + code[2] + 190687,
    // or can use brute force searching, after all the execution is fast.
}
