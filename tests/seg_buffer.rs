use crossbeam::scope;
use ripstruct::SegBuffer;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

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
    assert_eq!(results.len(), ITERATIONS / threads());
}

#[test]
fn from_iter() {
    let mut buffer: SegBuffer<_> = (0..ITERATIONS).into_iter().collect();

    for x in 0..ITERATIONS {
        assert_eq!(buffer.pop(), Some(x));
    }

    assert_eq!(buffer.pop(), None);
}

#[test]
fn no_double_drop() {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug, PartialEq)]
    struct Dropped(usize);

    impl Drop for Dropped {
        fn drop(&mut self) {
            COUNTER.fetch_add(self.0, Ordering::Relaxed);
        }
    }

    let mut buffer = SegBuffer::new();

    scope(|s| {
        for _ in 0..threads() {
            s.spawn(|_| {
                for x in 0..ITERATIONS {
                    buffer.push(Dropped(x));
                }
            });
        }
    })
    .unwrap();

    assert_eq!(COUNTER.load(Ordering::Relaxed), 0);

    for _ in 0..ITERATIONS * threads() {
        buffer.pop().unwrap();
    }

    assert_eq!(buffer.pop(), None);

    assert_eq!(
        COUNTER.load(Ordering::Relaxed),
        (0..ITERATIONS).sum::<usize>() * threads()
    );
}

#[test]
fn iter() {
    let mut buffer = SegBuffer::new();

    for x in 0..ITERATIONS {
        buffer.push(x);
    }

    for _ in 0..2 {
        buffer
            .iter()
            .enumerate()
            .for_each(|(i, x)| assert_eq!(i, *x));
    }

    let mut vec = vec![];
    vec.extend(buffer.iter_slices().flatten());

    vec.into_iter()
        .enumerate()
        .for_each(|(i, x)| assert_eq!(i, x));
}

#[test]
fn iter_mut() {
    let mut buffer = SegBuffer::new();

    for x in 0..ITERATIONS {
        buffer.push(x);
    }

    buffer.iter_mut().for_each(|x| *x *= 2);

    buffer
        .iter()
        .enumerate()
        .for_each(|(i, x)| assert_eq!(i * 2, *x));
}

#[cfg(feature = "rayon")]
#[cfg_attr(feature = "rayon", test)]
fn par_iter() {
    use rayon::prelude::*;

    let mut buffer = SegBuffer::new();

    for x in 0..ITERATIONS {
        buffer.push(x);
    }

    buffer.par_iter_mut().for_each(|x| *x *= 2);

    buffer
        .iter()
        .enumerate()
        .for_each(|(i, x)| assert_eq!(i * 2, *x));
}

fn threads() -> usize {
    num_cpus::get()
}
