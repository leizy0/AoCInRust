use std::sync::Arc;

use day2_5_7_9_11_13_15_17_19::{
    arcade::{ArcadeCabinet, AutoPlayer, Screen, TileId},
    int_code::{
        com::ParaIntCodeComputer,
        io::{Channel, ParaIODevice, ParaInputDevice, ParaOutputDevice},
        read_int_code,
    },
};
use rayon::ThreadPoolBuilder;

fn main() {
    let input_path = "day13_inputs.txt";
    let mut int_code = match read_int_code(input_path) {
        Ok(ic) => ic,
        Err(e) => {
            eprintln!(
                "Failed to read int code from file({}), get error({})",
                input_path, e
            );
            return;
        }
    };

    // Run arcade program to init settings(screen).
    let init_input = ParaInputDevice::new(Channel::new(&[]));
    let init_output = ParaOutputDevice::new(Channel::new(&[]));
    let mut computer = ParaIntCodeComputer::new(false);
    computer
        .execute_with_io(&int_code, init_input, init_output.clone())
        .expect("Failed to run arcade program for initialization");
    let screen = init_output
        .check(|c| Screen::from_ints(c.data().iter().copied()))
        .expect("Failed to init screen from arcade program.");

    // Run actual game in arcade program.
    // Insert coins
    int_code[0] = 2;
    let fps = 30;
    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("Failed to build thread pool.");
    let thread_pool = Arc::new(thread_pool);
    thread_pool.scope(|s| {
        s.spawn(|_s| {
            // let player = ManualPlayer::new(thread_pool.clone());
            let player = AutoPlayer::new();
            let mut cabinet = ArcadeCabinet::new(fps, screen, player, thread_pool.clone());
            cabinet.start().unwrap();
            let io_device = ParaIODevice::new(cabinet);
            match computer.execute_with_io(
                &int_code,
                io_device.input_device(),
                io_device.output_device(),
            ) {
                Ok(res) => io_device.tweak(|cab| {
                    cab.stop().unwrap();
                    let (block_n, score) =
                        cab.check_screen(|s| (s.count_id(TileId::Block), s.score()));
                    println!(
                        "After {} steps, game ends, {} blocks remained, the final score is {}.",
                        res.step_count(),
                        block_n,
                        score
                    );
                }),
                Err(e) => io_device.tweak(|cab| {
                    cab.stop().unwrap();
                    eprintln!("Failed to run game, get error({})", e);
                }),
            }
        });
    });
}
