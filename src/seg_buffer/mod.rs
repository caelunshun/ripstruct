use raw::RawBuffer;
use std::iter::FromIterator;

mod raw;
#[cfg(feature = "rayon")]
mod rayon;

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
}

impl<T> FromIterator<T> for SegBuffer<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let buffer = SegBuffer::new();

        iter.into_iter().for_each(|x| buffer.push(x));

        buffer
    }
}
