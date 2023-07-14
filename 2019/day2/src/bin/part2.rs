use day2::int_code::{execute_int_code, read_int_code};

fn main() {
    let int_code_file = "inputs.txt";
    let int_code = read_int_code(int_code_file)
        .expect(&format!("Failed to read int code from {}", int_code_file));

    let mut int_code_image = int_code.clone();
    for code1 in 0..3u32 {
        for code2 in 0..=9u32 {
            int_code_image.copy_from_slice(&int_code);
            int_code_image[1] = code1;
            int_code_image[2] = code2;
            match execute_int_code(&mut int_code_image) {
                Ok(step_count) => println!(
                    "{:>2} | {:>2}: After {} steps, program halts, code[0] = {}",
                    code1, code2, step_count, int_code_image[0]
                ),
                Err(e) => println!("Failed to run int code, get error({})", e),
            }
        }
    }

    // Using linear regression, after program halts, code[0] = code[1] * 300000 + code[2] + 190687,
    // or can use brute force searching, after all the execution is fast.
}
