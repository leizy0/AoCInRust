use day2_5_7_9_11_13_15_17_19_21_23_25::{
    day25::{self, DroidConsole},
    int_code::{
        self,
        com::{ProcessState, SeqIntCodeComputer},
        io::SeqIODevice,
    },
};

fn main() {
    let input_path = day25::check_args()
        .inspect_err(|e| eprintln!("Failed to read given input path, get error({}).", e))
        .unwrap();
    let intcode = int_code::read_int_code(&input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read intcode program from given input file({}), get error({}).",
                input_path, e
            )
        })
        .unwrap();

    let io_dev = SeqIODevice::new(DroidConsole::new());
    let mut computer = SeqIntCodeComputer::new(false);
    match computer.execute_with_io(&intcode, io_dev.input_device(), io_dev.output_device()) {
        Ok(res) => {
            println!(
                "After {} step(s), given intcode program stopped.",
                res.step_count()
            );
            match res.state() {
                state @ (ProcessState::Block | ProcessState::Halt) => {
                    println!("Intcode program finished with state({:?}).", state);
                }
                s => eprintln!("Unexpected program state({:?}) when finished.", s),
            }
        }
        Err(e) => eprintln!("Failed to run given intcode program, get error({}).", e),
    }
}
