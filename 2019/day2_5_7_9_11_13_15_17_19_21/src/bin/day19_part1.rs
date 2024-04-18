use day2_5_7_9_11_13_15_17_19_21::{
    beam::{Error, PointType, Scanner},
    int_code::read_int_code,
};

fn main() -> Result<(), Error> {
    let input_path = "day19_inputs.txt";
    let int_code = read_int_code(input_path).map_err(Error::IntCodeError)?;

    let mut scanner = Scanner::new(int_code);
    let scan_area_width = 50;
    let scan_area_hight = 50;
    let area = scanner.scan_area(0..scan_area_width, 0..scan_area_hight)?;
    println!(
        "Area((0, 0), ({}, {})) scaned:\n{}",
        scan_area_width - 1,
        scan_area_hight - 1,
        area
    );
    println!(
        "There are {} point(s) been affected by tractor beam in area({} x {}).",
        area.iter().filter(|pt| **pt == PointType::Pulled).count(),
        scan_area_width,
        scan_area_hight
    );
    Ok(())
}
