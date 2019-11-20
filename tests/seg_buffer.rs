use crossbeam::scope;
use ripstruct::SegBuffer;
use std::collections::HashMap;

const ITERATIONS: usize = 1_000_000;

#[test]
fn staircase() {
    let mut buffer = SegBuffer::new();

    let mut seg_size = 64;

    while seg_size < 1024 {
        (0..seg_size).for_each(|x| buffer.push(x));
        (0..seg_size).for_each(|x| assert_eq!(buffer.pop(), Some(x)));

        seg_size *= 2;
    }
}

#[test]
fn smoke() {
    let mut buffer = SegBuffer::new();

    for _ in 0..100 {
        assert_eq!(buffer.pop(), None);
    }

    buffer.push(2);
    assert_eq!(buffer.pop(), Some(2));
    assert_eq!(buffer.pop(), None);

    buffer.push(3);
    buffer.push(4);
    assert_eq!(buffer.pop(), Some(3));
    assert_eq!(buffer.pop(), Some(4));
    assert_eq!(buffer.pop(), None);

    for x in 0..61 {
        buffer.push(x);
        assert_eq!(buffer.pop(), Some(x));
    }

    assert_eq!(buffer.pop(), None);

    buffer.push(83);
    assert_eq!(buffer.pop(), Some(83));
    assert_eq!(buffer.pop(), None);

    for _ in 0..100 {
        assert_eq!(buffer.pop(), None);
    }
}

#[test]
fn single_thread() {
    let mut buffer = SegBuffer::new();

    for x in 0..ITERATIONS {
        buffer.push(x);
    }

    for x in 0..ITERATIONS {
        assert_eq!(buffer.pop(), Some(x));
    }
}

#[test]
fn multi_thread() {
    let mut buffer = SegBuffer::new();

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

    let mut results = HashMap::new();
    for _ in 0..ITERATIONS {
        let x = buffer.pop().unwrap();
        *results.entry(x).or_insert(0) += 1;
    }

    for x in 0..ITERATIONS / threads() {
        assert_eq!(results[&x], threads());
    }

    assert_eq!(buffer.pop(), None);
}

#[test]
fn from_iter() {
    let mut buffer: SegBuffer<_> = (0..ITERATIONS).into_iter().collect();

    for x in 0..ITERATIONS {
        assert_eq!(buffer.pop(), Some(x));
    }

    assert_eq!(buffer.pop(), None);
}

fn threads() -> usize {
    num_cpus::get()
}
