use day2_5_7_9_11_13_15_17_19_21::{
    int_code::{com::SeqIntCodeComputer, io::SeqIODevice, read_int_code},
    spring::{DetectMode, HullDetector, RRegister, SSInstruction, WRegister},
};

fn main() {
    let input_path = "day21_inputs.txt";
    let int_code = read_int_code(input_path)
        .inspect_err(|e| eprintln!("Failed to read intcode from given input, get error({}).", e))
        .unwrap();
    // (!A || !B || !C) && D
    let script = vec![
        SSInstruction::not(RRegister::A, WRegister::T),
        SSInstruction::not(RRegister::B, WRegister::J),
        SSInstruction::or(RRegister::T, WRegister::J),
        SSInstruction::not(RRegister::C, WRegister::T),
        SSInstruction::or(RRegister::T, WRegister::J),
        SSInstruction::and(RRegister::D, WRegister::J),
    ];
    let io_dev = SeqIODevice::new(HullDetector::new(&script, DetectMode::Walk));
    let mut computer = SeqIntCodeComputer::new(false);
    match computer.execute_with_io(&int_code, io_dev.input_device(), io_dev.output_device()) {
        Ok(res) => {
            println!(
                "After {} step(s), program stopped in state({:?}).",
                res.step_count(),
                res.state()
            );
            io_dev.check(|d| {
                if let Some(d) = d.damage() {
                    println!("The springdroid went across the hull using given script, and found total {} unit(s) of damage.", d);
                } else {
                    println!("Failed to make springdroid going across hull, get log:\n{}", d.log());
                }
            });
        }
        Err(e) => eprintln!("Failed to execute hull detection program, get error({})", e),
    }
}
