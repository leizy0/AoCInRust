use clap::Parser;
use day4::{
    BirthYearVad, CliArgs, ExpirYearVad, EyeColorVad, HairColorVad, HeightVad, IssueYearVad,
    PassportIDVad, PropValidator,
};

fn main() {
    let args = CliArgs::parse();
    let passports = day4::read_pp(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read passports from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();

    let byr_vad = BirthYearVad {};
    let iyr_vad = IssueYearVad {};
    let eyr_vad = ExpirYearVad {};
    let hgt_vad = HeightVad {};
    let hcl_vad = HairColorVad {};
    let ecl_vad = EyeColorVad {};
    let pid_vad = PassportIDVad {};
    let prop_validators: [&dyn PropValidator; 7] = [
        &byr_vad, &iyr_vad, &eyr_vad, &hgt_vad, &hcl_vad, &ecl_vad, &pid_vad,
    ];
    let valid_counts = passports
        .iter()
        .filter(|pp| {
            prop_validators
                .iter()
                .all(|vad| pp.prop(vad.name()).is_some_and(|s| vad.validate(s)))
        })
        .count();
    println!(
        "There are {} valid passports in given {} records after using value validation.",
        valid_counts,
        passports.len()
    );
}
