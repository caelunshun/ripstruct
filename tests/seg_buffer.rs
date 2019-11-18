use crossbeam::scope;
use ripstruct::SegBuffer;

const ITERATIONS: usize = 1_000_000;

#[test]
fn single_thread() {
    let buffer = SegBuffer::new();

    for x in 0..ITERATIONS {
        buffer.push(x);
    }
}

#[test]
fn multi_thread() {
    let buffer = SegBuffer::new();

    scope(|s| {
        for _ in 0..threads() {
            s.spawn(|_| {
                for x in 0..ITERATIONS / threads() {
                    buffer.push(x);
                }
            });
        }
    })
    .unwrap();
}

fn threads() -> usize {
    num_cpus::get()
}
