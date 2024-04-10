use day2_5_7_9_11_13_15_17_19::{
    beam::{Error, Scanner},
    int_code::read_int_code,
};

fn main() -> Result<(), Error> {
    let input_path = "day19_inputs.txt";
    let int_code = read_int_code(input_path).map_err(Error::IntCodeError)?;

    let mut scanner = Scanner::new(int_code);
    let block_width = 100;
    let block_height = 100;
    let (top_left_p, bottom_right_p) = scanner.scan_first_block(block_width, block_height)?;
    println!(
        "The first block({} x {}) in beam is at ({}, {}).",
        block_width, block_height, top_left_p, bottom_right_p
    );
    Ok(())
}
