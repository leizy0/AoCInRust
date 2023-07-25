use day2_5_7::amp::{amp_loop, AmpSettings};
use day2_5_7::int_code::com::IntCodeComputer;
use day2_5_7::int_code::read_int_code;

fn main() {
    let init_amp_setting = (5..10).collect::<Vec<i32>>();
    let input_path = "day7_inputs.txt";
    let int_code = read_int_code(input_path).expect(&format!(
        "Failed to read int code from file({})",
        input_path
    ));
    let mut max_output_signal = i32::MIN;
    let mut max_output_setting = vec![-1; init_amp_setting.len()];
    let mut computer = IntCodeComputer::new(false);
    let mut try_count = 0;

    for setting in AmpSettings::from(init_amp_setting.as_slice()).iter() {
        let output_signal = match amp_loop(&mut computer, &int_code, &setting) {
            Ok(i) => i,
            Err(e) => {
                eprintln!(
                    "Failed to run amplifier loop with setting({:?}), get error({})",
                    &setting, e
                );
                continue;
            }
        };

        print!(
            "Try #{:>4}: Amplifier loop with setting({:?}) get output {}, ",
            try_count, &setting, output_signal
        );
        if output_signal > max_output_signal {
            max_output_signal = output_signal;
            max_output_setting = Vec::from(setting);
            println!("it's MAXIMUM by now.");
        } else {
            println!("trivial result.");
        }

        try_count += 1;
    }

    println!(
        "The maximum output signal({}) of amplifier loop can be achived by setting({:?})",
        max_output_signal, max_output_setting
    );
}
