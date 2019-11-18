use raw::RawBuffer;

mod raw;

/// An unbounded concurrent buffer implementation based on a linked list of segments.
///
/// This type is different from a queue in that it supports either writing
/// to the buffer in parallel xor reading from it sequentially. As a result,
/// it is faster than e.g.`crossbeam::SegQueue` for cases where the buffer
/// does not need to be read and written to at the same time.
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

    /// Pushes a value onto the end of the buffer.
    pub fn push(&self, value: T) {
        unsafe {
            self.raw.push(value);
        }
    }
}
