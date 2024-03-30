use day16::{read_signal, Error, FFT};

fn main() -> Result<(), Error> {
    let input_path = "inputs.txt";
    let signal = read_signal(input_path).expect(&format!(
        "Failed to read input signal from file({}).",
        input_path
    ));

    let output_offset = signal.iter().take(7).fold(0, |acc, ele| acc * 10 + ele);
    let phase_count = 100;
    let rep_count = 10000;
    let mut signal = rep_signal(&signal, rep_count);
    let fft = FFT::new(signal.len());
    fft.process_n(&mut signal, phase_count)?;

    let offset_eight_digits = signal
        .iter()
        .skip(output_offset as usize)
        .take(8)
        .map(|d| char::from_digit(*d, 10).unwrap())
        .collect::<String>();
    println!(
        "After {} phases, the offset({}) eight signal digits are {}",
        phase_count, output_offset, offset_eight_digits
    );

    Ok(())
}

fn rep_signal(signal: &[u32], rep_count: usize) -> Vec<u32> {
    let mut res = Vec::with_capacity(signal.len() * rep_count);
    for _ in 0..rep_count {
        res.extend(signal)
    }

    res
}
