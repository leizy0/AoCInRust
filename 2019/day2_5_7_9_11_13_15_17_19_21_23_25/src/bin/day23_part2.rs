use day2_5_7_9_11_13_15_17_19_21_23_25::{day23, int_code};

fn main() {
    let input_path = day23::check_args()
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
    let nat_addr = 255;
    let nat_send_addr = 0;
    match day23::run_network_nat(host_n, &intcode, nat_addr, nat_send_addr) {
        Ok(hub) => {
            let nat_sent_pacs = hub.nat().unwrap().sent_pacs();
            let nat_sent_pacs_n = nat_sent_pacs.len();
            assert!(nat_sent_pacs_n > 2);
            let nat_last_sent_y = nat_sent_pacs[nat_sent_pacs_n - 1].y();
            assert!(nat_last_sent_y == nat_sent_pacs[nat_sent_pacs_n - 2].y());
            println!(
                "NAT has sent two packets with this same y({}) to address {} in a row.",
                nat_last_sent_y, nat_send_addr
            );
        }
        Err(e) => {
            eprintln!("Failed to run the whole network to end, get error({}).", e);
        }
    }
}
