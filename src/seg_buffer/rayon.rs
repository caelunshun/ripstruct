use crate::seg_buffer::raw::ParRawIter;
use crate::SegBuffer;
use rayon::iter::plumbing::{Consumer, Folder, UnindexedConsumer, UnindexedProducer};
use rayon::iter::{plumbing, Flatten};
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

/// A parallel iterator over slices in a `SegBuffer`.
pub struct ParSliceIter<'a, T> {
    pub(super) raw: ParRawIter<'a, T>,
}

impl<'a, T> ParallelIterator for ParSliceIter<'a, T>
where
    T: Send + Sync,
{
    type Item = &'a [T];

    fn drive_unindexed<C>(self, consumer: C) -> <C as Consumer<Self::Item>>::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        plumbing::bridge_unindexed(self, consumer)
    }
}

impl<'a, T> UnindexedProducer for ParSliceIter<'a, T>
where
    T: Send,
{
    type Item = &'a [T];

    fn split(self) -> (Self, Option<Self>) {
        let (old, new) = self.raw.split();

        (Self { raw: old }, new.map(|raw| Self { raw }))
    }

    fn fold_with<F>(self, folder: F) -> F
    where
        F: Folder<Self::Item>,
    {
        let slice: &'a [T] = self.raw.slice();

        folder.consume(slice)
    }
}

/// A parallel iterator over mutable slices in a `SegBuffer`.
pub type ParSliceIterMut<'a, T> = ParRawIter<'a, T>;

/// A parallel iterator over references to values in a `SegBuffer`.
pub type ParIter<'a, T> = Flatten<ParSliceIter<'a, T>>;

/// A parallel iterator over mutable references to values in a `SegBuffer`.
pub type ParIterMut<'a, T> = Flatten<ParSliceIterMut<'a, T>>;

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
