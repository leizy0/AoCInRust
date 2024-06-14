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
    if nums.len() <= ADDEND_LEN {
        println!("Given numbers({} in total) aren't enough to be encrypted data using XMAS, at least {} numbers are expected.", nums.len(), ADDEND_LEN + 1);
        return;
    }

    let mut addends = Vec::from_iter(nums.iter().take(ADDEND_LEN).copied());
    for sum_ind in ADDEND_LEN..nums.len() {
        let sum = nums[sum_ind];
        addends.sort_unstable();
        let mut is_valid = false;
        for addend in &addends {
            if *addend > sum / 2 {
                break;
            }

            let expect_n = sum - *addend;
            if let Ok(_) = addends.binary_search(&expect_n) {
                is_valid = true;
                break;
            }
        }

        if !is_valid {
            println!("The first invalid value according to XMAS rule is {}.", sum);
            return;
        }

        let first_addend_ind = addends.binary_search(&nums[sum_ind - ADDEND_LEN]).unwrap();
        addends[first_addend_ind] = sum;
    }

    println!(
        "There isn't any invalid value according to XMAS rule in given numbers({} in total)",
        nums.len()
    );
}
