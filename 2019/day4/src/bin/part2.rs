use day4::code::maybe_is_code2;

fn main() {
    let code_range = 234208..=765869;
    let maybe_code_count = code_range.filter(|&n| maybe_is_code2(n)).count();
    println!(
        "There are {} candidate numbers may be the code.",
        maybe_code_count
    );
}
