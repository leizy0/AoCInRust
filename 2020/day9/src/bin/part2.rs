use clap::Parser;
use day9::CliArgs;

fn main() {
    let args = CliArgs::parse();
    let nums = day9::read_num(&args.input_path)
        .inspect_err(|e| {
            eprintln!(
                "Failed to read numbers from given input file({}), get error({}).",
                args.input_path.display(),
                e
            )
        })
        .unwrap();
    const ADDEND_LEN: usize = 25;
    let nums_n = nums.len();
    if nums_n <= ADDEND_LEN {
        println!("Given numbers({} in total) aren't enough to be encrypted data using XMAS, at least {} numbers are expected.", nums.len(), ADDEND_LEN + 1);
        return;
    }

    if let Some(invalid_value) = day9::invalid_xmax_v(&nums, ADDEND_LEN) {
        let mut start_ind = 0;
        let mut stop_ind = 0;
        let mut sum = 0;
        while stop_ind <= nums_n {
            // Extend to end if sum is smaller than target.
            while sum < invalid_value && stop_ind < nums_n {
                sum += nums[stop_ind];
                stop_ind += 1;
            }

            // Shrink from begining if sum is larger than target.
            while sum > invalid_value && start_ind < stop_ind {
                sum -= nums[start_ind];
                start_ind += 1;
            }

            if sum == invalid_value {
                // Found set which sum exactly to target.
                let addend_set = &nums[start_ind..stop_ind];
                let min = addend_set.iter().min().unwrap();
                let max = addend_set.iter().max().unwrap();
                let weakness = min + max;
                println!("The continuous set of numbers which sum to the first invalid value({}) is {:?}, so the encryption weakness is {}.", invalid_value, addend_set, weakness);
                return;
            }
        }

        println!(
            "There's no continuous set of numbers which sum to the first invalid value({}).",
            invalid_value
        );
    } else {
        println!(
            "There isn't any invalid value according to XMAS rule in given numbers({} in total)",
            nums.len()
        );
    }
}
