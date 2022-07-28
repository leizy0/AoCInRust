use day16::inst;

fn main() {
    let input_path = "samples.txt";
    let samples = inst::load_samples(input_path)
        .expect(&format!("Failed to load samples from input file({})", input_path));
    let mut confuse_sample_count = 0;
    for sample in samples {
        let guess_insts = inst::guess_insts(&sample);
        if guess_insts.len() >= 3 {
            confuse_sample_count += 1;
        }
    }

    println!("Found {} sample has 3 or more possible instruction opcodes", confuse_sample_count);
}
