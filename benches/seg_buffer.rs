#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion};
use crossbeam::queue::{ArrayQueue, SegQueue};
use ripstruct::SegBuffer;

const AMOUNT: usize = 1_000;

fn seg_buffer_push_single_thread(c: &mut Criterion) {
    c.bench_function("buffer_push_single_thread", |b| {
        b.iter_batched(
            || SegBuffer::new(),
            |buffer| {
                for x in 0..AMOUNT {
                    buffer.push(x);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

fn seg_queue_push_single_thread(c: &mut Criterion) {
    c.bench_function("queue_push_single_thread", |b| {
        b.iter_batched(
            || SegQueue::new(),
            |queue| {
                for x in 0..AMOUNT {
                    queue.push(x);
                }
            },
            BatchSize::SmallInput,
        )
    });
}

fn array_queue_push_single_thread(c: &mut Criterion) {
    c.bench_function("array_queue_push_single_thread", |b| {
        b.iter_batched(
            || ArrayQueue::new(AMOUNT),
            |queue| {
                for x in 0..AMOUNT {
                    queue.push(x).unwrap();
                }
            },
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(
    single_thread,
    seg_buffer_push_single_thread,
    seg_queue_push_single_thread,
    array_queue_push_single_thread,
);
criterion_main!(single_thread);
