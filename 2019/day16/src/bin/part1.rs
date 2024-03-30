use day16::{read_signal, Error, FFT};

fn main() -> Result<(), Error> {
    let input_path = "inputs.txt";
    let mut signal = read_signal(input_path).expect(&format!(
        "Failed to read input signal from file({}).",
        input_path
    ));

    let phase_count = 100;
    let fft = FFT::new(signal.len());
    fft.process_n(&mut signal, phase_count)?;

    let first_eight_digits = signal
        .iter()
        .take(8)
        .map(|d| char::from_digit(*d, 10).unwrap())
        .collect::<String>();
    println!(
        "After {} phases, the first eight signal digits are {}",
        phase_count, first_eight_digits
    );

    Ok(())
}
