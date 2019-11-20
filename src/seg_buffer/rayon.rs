use crate::SegBuffer;
use rayon::prelude::*;

impl<T> FromParallelIterator<T> for SegBuffer<T>
where
    T: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        let buffer = SegBuffer::new();

        par_iter.into_par_iter().for_each(|x| buffer.push(x));

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn from_par_iter() {
        let mut buffer: SegBuffer<i32> = (0..1_000_000).into_par_iter().collect();

        let mut results = HashSet::new();
        for _ in 0..1_000_000 {
            results.insert(buffer.pop().unwrap());
        }

        assert_eq!(buffer.pop(), None);

        for x in 0..1_000_000 {
            assert!(results.contains(&x));
        }

        assert_eq!(results.len(), 1_000_000);
    }
}
