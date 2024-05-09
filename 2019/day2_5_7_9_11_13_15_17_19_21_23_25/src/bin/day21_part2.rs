use day2_5_7_9_11_13_15_17_19_21_23_25::{
    day21::{DetectMode, HullDetector, RRegister, SSInstruction, WRegister},
    int_code::{com::SeqIntCodeComputer, io::SeqIODevice, read_int_code},
};

fn main() {
    let input_path = "day21_inputs.txt";
    let int_code = read_int_code(input_path)
        .inspect_err(|e| eprintln!("Failed to read intcode from given input, get error({}).", e))
        .unwrap();
    // (!A || !B || (A && B && !C && (E || H))) && D
    let script = vec![
        SSInstruction::or(RRegister::E, WRegister::T),
        SSInstruction::or(RRegister::H, WRegister::T),
        SSInstruction::not(RRegister::C, WRegister::J),
        SSInstruction::and(RRegister::T, WRegister::J),
        SSInstruction::and(RRegister::A, WRegister::J),
        SSInstruction::and(RRegister::B, WRegister::J),
        SSInstruction::not(RRegister::A, WRegister::T),
        SSInstruction::or(RRegister::T, WRegister::J),
        SSInstruction::not(RRegister::B, WRegister::T),
        SSInstruction::or(RRegister::T, WRegister::J),
        SSInstruction::and(RRegister::D, WRegister::J),
    ];
    let io_dev = SeqIODevice::new(HullDetector::new(&script, DetectMode::Run));
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
