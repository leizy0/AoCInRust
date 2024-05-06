use day2_5_7_9_11_13_15_17_19_21_23_25::{int_code, nic};

fn main() {
    let input_path = nic::check_args()
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
    let host_n = 50;
    match nic::run_network(host_n, &intcode) {
        Ok(hub) => {
            let check_addr = 255;
            if let Some(packet) = hub.recv_pac_log(check_addr, 0) {
                println!(
                    "The first packet sent to address({}) is {}",
                    check_addr, packet
                );
            } else {
                eprintln!("There isn't any packet sent to address({}).", check_addr);
            }
        }
        Err(e) => {
            eprintln!("Failed to run the whole network to end, get error({}).", e);
        }
    }
}
