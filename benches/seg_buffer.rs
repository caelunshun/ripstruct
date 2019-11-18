#[macro_use]
extern crate criterion;

use criterion::{BatchSize, Criterion};
use crossbeam::queue::{ArrayQueue, SegQueue};
use crossbeam::scope;
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

fn seg_buffer_push_concurrent(c: &mut Criterion) {
    c.bench_function("seg_buffer_push_concurrent", |b| {
        b.iter_batched(
            || SegBuffer::new(),
            |buffer| {
                scope(|s| {
                    for _ in 0..threads() {
                        s.spawn(|_| {
                            for x in 0..AMOUNT / threads() {
                                buffer.push(x);
                            }
                        });
                    }
                })
                .unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

fn seg_queue_push_concurrent(c: &mut Criterion) {
    c.bench_function("seg_queue_push_concurrent", |b| {
        b.iter_batched(
            || SegQueue::new(),
            |queue| {
                scope(|s| {
                    for _ in 0..threads() {
                        s.spawn(|_| {
                            for x in 0..AMOUNT / threads() {
                                queue.push(x);
                            }
                        });
                    }
                })
                .unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

fn array_queue_push_concurrent(c: &mut Criterion) {
    c.bench_function("array_queue_push_concurrent", |b| {
        b.iter_batched(
            || ArrayQueue::new(AMOUNT),
            |queue| {
                scope(|s| {
                    for _ in 0..threads() {
                        s.spawn(|_| {
                            for x in 0..AMOUNT / threads() {
                                queue.push(x).unwrap();
                            }
                        });
                    }
                })
                .unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

fn thread_creation(c: &mut Criterion) {
    c.bench_function("thread_creation", |b| {
        b.iter(|| {
            scope(|s| {
                for _ in 0..threads() {
                    s.spawn(|_| ());
                }
            })
            .unwrap();
        });
    });
}

fn threads() -> usize {
    num_cpus::get()
}

criterion_group!(
    single_thread,
    seg_buffer_push_single_thread,
    seg_queue_push_single_thread,
    array_queue_push_single_thread,
);
criterion_group!(
    concurrent,
    thread_creation,
    seg_buffer_push_concurrent,
    seg_queue_push_concurrent,
    array_queue_push_concurrent,
);
criterion_main!(single_thread, concurrent);
