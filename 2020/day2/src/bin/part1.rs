use clap::Parser;
use day2::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let pws = day2::read_pws(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read passwords from given input file({}), get error({}).",
                args.input_path, e
            )
        })
        .unwrap();
    
    let valid_count = pws.iter().filter(|pw| pw.is_valid()).count();

    println!("There are {} valid passwords in total.", valid_count);
}
