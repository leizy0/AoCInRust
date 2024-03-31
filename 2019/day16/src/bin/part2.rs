use day16::{read_signal, Error, PartialFFT, RepeatSignal};

fn main() -> Result<(), Error> {
    let input_path = "inputs.txt";
    let signal = read_signal(input_path).expect(&format!(
        "Failed to read input signal from file({}).",
        input_path
    ));

    let output_offset = signal
        .iter()
        .take(7)
        .fold(0, |acc, ele| acc * 10 + (*ele as usize));
    let phase_count = 100;
    let rep_count = 10000;
    let signal = RepeatSignal::new(&signal, rep_count);
    let pfft = PartialFFT::new(output_offset, &signal, phase_count);

    let offset_eight_digits = (output_offset..)
        .take(8)
        .map(|ind| pfft.nth_ele(ind).map(|e| char::from_digit(e, 10).unwrap()))
        .collect::<Result<String, Error>>()?;
    println!(
        "After {} phases, the offset({}) eight signal digits are {}.",
        phase_count, output_offset, offset_eight_digits
    );

    Ok(())
}
