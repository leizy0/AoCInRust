use day2_5_7_9_11_13_15_17_19_21_23::{
    ascii::{self, Error, ScaffoldMap, ZipPathPilot, ZipRobotPath},
    int_code::{
        com::SeqIntCodeComputer,
        io::{Channel, SeqIODevice, SeqInputDevice, SeqOutputDevice},
        read_int_code,
    },
};

fn main() -> Result<(), Error> {
    let input_path = "day17_inputs.txt";
    let mut int_code = read_int_code(input_path).map_err(Error::IntCodeError)?;

    let input_dev = SeqInputDevice::new(Channel::new(&[]));
    let output_dev = SeqOutputDevice::new(Channel::new(&[]));
    let mut computer = SeqIntCodeComputer::new(false);
    let one_touch_paths = computer
        .execute_with_io(&int_code, input_dev, output_dev.clone())
        .map_err(Error::IntCodeError)
        .and_then(|res| {
            println!(
                "After {} steps, the ASCII program stopped.",
                res.step_count()
            );
            output_dev.check(|ap| {
                let scaffold_map = ScaffoldMap::try_from_ints(ap.data().iter().copied())?;
                println!("Get scaffold map:\n{}", scaffold_map);
                Ok(scaffold_map.one_touch_paths())
            })
        })?;

    println!(
        "Found {} one touch paths in given scaffold map.",
        one_touch_paths.len()
    );

    for (ind, path) in one_touch_paths.iter().enumerate() {
        println!("Zipping path # {}: {}", ind, path);
        let zip_path = ZipRobotPath::new(path, |path| ascii::text_len(path) <= 20);
        println!("Main move function: {:?}", zip_path.path());
        for (ind, sub_path) in zip_path.sub_paths().iter().enumerate() {
            println!("Sub move function #{} : {}", ind, sub_path);
        }

        if zip_path.sub_paths().len() <= 3 {
            // Run intcode program again, control vaccum robot to traverse the map this time.
            int_code[0] = 2;
            let io_dev = SeqIODevice::new(ZipPathPilot::new(&zip_path, false));
            computer
                .execute_with_io(&int_code, io_dev.input_device(), io_dev.output_device())
                .and_then(|res| {
                    println!(
                        "After {} steps, traversal with this zip path stops.",
                        res.step_count()
                    );
                    io_dev.check(|pilot| {
                        println!(
                            "Get {} units of space dust in the traversal.",
                            pilot.dust_n()
                        )
                    });
                    Ok(())
                })
                .map_err(Error::IntCodeError)?;
            break;
        }

        println!("");
    }

    Ok(())
}
