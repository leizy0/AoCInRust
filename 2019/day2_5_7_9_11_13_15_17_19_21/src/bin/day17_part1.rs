use day2_5_7_9_11_13_15_17_19_21::{
    ascii::{Error, ScaffoldMap},
    int_code::{
        com::SeqIntCodeComputer,
        io::{Channel, SeqInputDevice, SeqOutputDevice},
        read_int_code,
    },
};

fn main() -> Result<(), Error> {
    let input_path = "day17_inputs.txt";
    let int_code = read_int_code(input_path).map_err(Error::IntCodeError)?;

    let input_dev = SeqInputDevice::new(Channel::new(&[]));
    let output_dev = SeqOutputDevice::new(Channel::new(&[]));
    let mut computer = SeqIntCodeComputer::new(false);
    computer
        .execute_with_io(&int_code, input_dev, output_dev.clone())
        .map_err(Error::IntCodeError)
        .and_then(|res| {
            println!(
                "After {} steps, the ASCII program stopped.",
                res.step_count()
            );
            output_dev.check(|ap| {
                let scaffold_map = ScaffoldMap::try_from_ints(ap.data().iter().copied())?;
                let intersections = scaffold_map.intersections();
                let align_para_sum: usize = intersections.iter().map(|pos| pos.x() * pos.y()).sum();
                println!(
                    "There are {} intersctions, the sum of the alignment parameters is {}.",
                    intersections.len(),
                    align_para_sum
                );
                Ok(())
            })
        })
}
