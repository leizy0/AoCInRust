use criterion::{Criterion, criterion_group, criterion_main};
use day12::n_body::{read_n_body, NBodySimulator};

pub fn step_benchmark(c: &mut Criterion) {
    let input_path = "test_inputs1.txt";
    let init_bodies = match read_n_body(input_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "Failed to read n-body setting from input file({}), get error({})",
                input_path, e
            );
            return;
        }
    };

    let mut simulator = NBodySimulator::new(init_bodies);
    c.bench_function("simulator step 10000", |b| { b.iter(|| {
        for _ in 0..10000 {
            simulator.step();
        }
    })});
}

criterion_group!(simulator_benches, step_benchmark);
criterion_main!(simulator_benches);
