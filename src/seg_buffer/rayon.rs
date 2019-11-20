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
        let mut buffer = SegBuffer::new();
        let writer = buffer.writer();

        par_iter.into_par_iter().for_each(|x| writer.push(x));

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_par_iter() {
        let _buffer: SegBuffer<i32> = (0..1_000_000).into_par_iter().collect();
    }
}
