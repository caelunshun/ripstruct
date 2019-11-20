use crate::seg_buffer::raw::RawIter;
use raw::RawBuffer;
use std::iter::{Flatten, FromIterator};

mod raw;
#[cfg(feature = "rayon")]
mod rayon;

#[cfg(feature = "rayon")]
pub use self::rayon::*;
#[cfg(feature = "rayon")]
use ::rayon::iter::ParallelIterator;

/// An unbounded, wait-free buffer implemented using a linked list of segments.
///
/// Akin to `crossbeam::SegQueue`, this data structure acts as a queue of values.
/// However `SegBuffer` is more specialized in that it only supports _either_ reading
/// or writing values (although multiple threads may write at the same time).
/// This property allows `SegBuffer` to be faster than `SegQueue`—often as much as twice
/// as performant.
///
/// Additionally, `SegBuffer` supports a variety of additional operations which `SegQueue`
/// lacks. These include:
/// * Iterating over elements in the buffer, using either normal iterators or Rayon parallel iterators.
/// * Efficient O(1) pushing of a vector of elements at once.
/// * Direct slice access to the inner values.p
pub struct SegBuffer<T> {
    raw: RawBuffer<T>,
}

impl<T> Default for SegBuffer<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SegBuffer<T> {
    /// Creates a new, empty `SegBuffer<T>`.
    pub fn new() -> Self {
        Self {
            raw: RawBuffer::new(),
        }
    }

    /// Pushes an element to the buffer.
    pub fn push(&self, value: T) {
        unsafe { self.raw.push(value) }
    }

    /// Pops an element from the back of the buffer.
    pub fn pop(&mut self) -> Option<T> {
        unsafe { self.raw.pop() }
    }

    /// Returns an iterator over slices in the buffer in order.
    pub fn iter_slices(&mut self) -> SliceIter<T> {
        SliceIter {
            raw: self.raw.iter(),
        }
    }

    /// Returns an iterator over mutable slices in the buffer in order.
    pub fn iter_slices_mut(&mut self) -> SliceIterMut<T> {
        SliceIterMut {
            raw: self.raw.iter(),
        }
    }

    /// Returns an iterator over references to values in the buffer in order.
    pub fn iter(&mut self) -> Iter<T> {
        self.iter_slices().flatten()
    }

    /// Returns an iterator over mutable references to values in the buffer in order.
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.iter_slices_mut().flatten()
    }

    /// Returns a parallel iterator over slices in the buffer in order.
    #[cfg(feature = "rayon")]
    pub fn par_iter_slices(&mut self) -> ParSliceIter<T>
    where
        T: Send,
    {
        ParSliceIter {
            raw: self.raw.par_iter(),
        }
    }

    /// Returns a parallel iterator over mutable slices in the buffer in order.
    #[cfg(feature = "rayon")]
    pub fn par_iter_slices_mut(&mut self) -> ParSliceIterMut<T>
    where
        T: Send,
    {
        self.raw.par_iter()
    }

    /// Returns a parallel iterator over references to values in the buffer in order.
    #[cfg(feature = "rayon")]
    pub fn par_iter(&mut self) -> ParIter<T>
    where
        T: Send + Sync,
    {
        <ParSliceIter<T> as ParallelIterator>::flatten(self.par_iter_slices())
    }

    /// Returns a parallel iterator over mutable references to values in the buffer in order.
    #[cfg(feature = "rayon")]
    pub fn par_iter_mut(&mut self) -> ParIterMut<T>
    where
        T: Send,
    {
        <ParSliceIterMut<T> as ParallelIterator>::flatten(self.par_iter_slices_mut())
    }
}

impl<T> FromIterator<T> for SegBuffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let buffer = SegBuffer::new();

        iter.into_iter().for_each(|x| buffer.push(x));

        buffer
    }
}

pub struct SliceIter<'a, T> {
    raw: RawIter<'a, T>,
}

impl<'a, T> Iterator for SliceIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        match self.raw.next() {
            Some(slice) => Some(slice),
            None => None,
        }
    }
}

pub struct SliceIterMut<'a, T> {
    raw: RawIter<'a, T>,
}

impl<'a, T> Iterator for SliceIterMut<'a, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.raw.next()
    }
}

pub type Iter<'a, T> = Flatten<SliceIter<'a, T>>;
pub type IterMut<'a, T> = Flatten<SliceIterMut<'a, T>>;
