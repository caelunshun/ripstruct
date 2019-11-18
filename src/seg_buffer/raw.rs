use std::cell::UnsafeCell;
use std::cmp::min;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::{iter, ptr};

/// Capacity of the first segment in the buffer.
const STARTING_SIZE: usize = 64;

/// A segment in the buffer.
struct Segment<T> {
    /// Pointer to the next segment in the list, or `ptr::null`
    /// if it does not exist.
    next: AtomicPtr<Segment<T>>,
    /// Pointer to the previous segment in the list, or `ptr::null`
    /// if it does not exist.
    prev: AtomicPtr<Segment<T>>,
    /// Capacity of this segment.
    capacity: usize,
    /// Index of the next value to write into this
    /// segment. Note that this value may exceed
    /// `capacity`: if so, the buffer has been fully written.
    front: AtomicUsize,
    /// Index of the next value to read from this
    /// segment.
    ///
    /// If the value is greater than or equal to `capacity`,
    /// then the entire segment has been read.
    /// If the value is greater than or equal to `front`,
    /// then there are no values in this segment remaining
    /// to read.
    back: AtomicUsize,
    /// Array of values in this segment.
    ///
    /// This array has length `self.capacity`.
    array: Vec<UnsafeCell<MaybeUninit<T>>>,
}

impl<T> Drop for Segment<T> {
    fn drop(&mut self) {
        let front = min(self.capacity, *self.front.get_mut());
        let back = min(self.capacity, *self.back.get_mut());

        for i in back..front {
            unsafe {
                drop(ptr::read((&mut *self.array[i].get()).as_mut_ptr()));
            }
        }
    }
}

pub struct RawBuffer<T> {
    /// Pointer to the head segment.
    ///
    /// Do note that segments may exist past this
    /// head as linked by the `next` pointer.
    /// The head is only the segment to which values
    /// are currently written.
    ///
    /// This value must never be null.
    head: AtomicPtr<Segment<T>>,
    /// Pointer to the tail segment.
    ///
    /// This value must never be null.
    tail: AtomicPtr<Segment<T>>,
}

impl<T> RawBuffer<T> {
    pub fn new() -> Self {
        let head = new_segment(STARTING_SIZE);
        Self {
            head: AtomicPtr::new(head),
            tail: AtomicPtr::new(head),
        }
    }

    /// Pushes a value onto the buffer.
    ///
    /// # Safety
    /// Only other calls to `push` may execute concurrently.
    pub unsafe fn push(&self, value: T) {
        // Obtain a position in a segment.
        let (segment, index) = loop {
            let head = &mut *self.head.load(Ordering::Acquire);

            let position = head.front.fetch_add(1, Ordering::AcqRel);

            if position < head.capacity {
                break (head, position);
            } else {
                // Position is past the end of the segment. We do the following:
                // * If `head->next` is set, then there is another segment available.
                // Attempt to set it as the new head and continue the loop.
                // * Otherwise, there are no more available segments.
                // We allocate a new one and traverse the list forward
                // until there is a segment whose `next` pointer we can
                // set to the new segment (i.e. the old value isn't null).
                let next = head.next.load(Ordering::Acquire);
                if !next.is_null() {
                    self.head
                        .compare_and_swap(head as *mut _, next, Ordering::AcqRel);
                } else {
                    // Allocate new segment.
                    let new_segment = new_segment(head.capacity * 2);

                    // Traverse to the end of the list and add the new segment.
                    let mut head = head as *mut Segment<T>;
                    let mut next = ptr::null_mut();
                    while !head.is_null() && {
                        next = (&*head).next.compare_and_swap(
                            ptr::null_mut(),
                            new_segment,
                            Ordering::AcqRel,
                        );
                        !next.is_null()
                    } {
                        head = next;
                    }
                }
            }
        };

        // Write value into segment.
        let ptr = (&mut *segment.array[index].get()).as_mut_ptr();

        ptr::write(ptr, value);
    }

    /// Removes a value from the start of the buffer.
    ///
    /// # Safety
    /// Neither other pop operations nor push operations may
    /// run in parallel with this function.
    pub unsafe fn pop(&self) -> Option<T> {
        unimplemented!()
    }
}

impl<T> Drop for RawBuffer<T> {
    fn drop(&mut self) {
        let mut tail = *self.tail.get_mut();

        while !tail.is_null() {
            unsafe {
                let temp = *(&mut *tail).next.get_mut();
                drop(Box::from_raw(tail));
                tail = temp;
            }
        }
    }
}

fn new_segment<T>(capacity: usize) -> *mut Segment<T> {
    let boxed = Box::new(Segment {
        next: AtomicPtr::new(ptr::null_mut()),
        prev: AtomicPtr::new(ptr::null_mut()),
        capacity,
        front: AtomicUsize::new(0),
        back: AtomicUsize::new(0),
        array: iter::repeat_with(|| UnsafeCell::new(MaybeUninit::uninit()))
            .take(capacity)
            .collect(),
    });

    Box::into_raw(boxed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let buffer = RawBuffer::new();

        for i in 0..1024 {
            unsafe {
                buffer.push(i);
                //assert_eq!(buffer.pop_back(), Some(i));
            }
        }

        for i in 0..65536 {
            unsafe { buffer.push(i) };
        }

        for i in (0..65536).rev() {
            //assert_eq!(unsafe { buffer.pop_front() }, Some(i));
        }
    }
}
