use ahash::ABuildHasher;
use arrayvec::ArrayVec;
use epoch::{Atomic, Guard, Shared};
use std::cell::UnsafeCell;
use std::hash::{BuildHasher, Hash};
use std::iter;
use std::hash::Hasher;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

const GROUP_SIZE: usize = 16;

pub struct Group<K, V> {
    controls: [AtomicU64; GROUP_SIZE / 8],
    keys: [UnsafeCell<MaybeUninit<K>>; GROUP_SIZE],
    values: [UnsafeCell<MaybeUninit<Atomic<V>>>; GROUP_SIZE],
    inserting: AtomicBool,
}

impl<K, V> Group<K, V> {
    pub fn new() -> Self {
        Self {
            controls: Default::default(),
            keys: iter::repeat_with(|| UnsafeCell::new(MaybeUninit::uninit()))
                .take(GROUP_SIZE)
                .collect::<ArrayVec<[_; GROUP_SIZE]>>()
                .inner()
                .unwrap(),
            values: iter::repeat_with(|| MaybeUninit::uninit())
                .take(GROUP_SIZE)
                .collect::<ArrayVec<[_; GROUP_SIZE]>>()
                .inner()
                .unwrap(),
            inserting: AtomicBool::new(false),
        }
    }
}

pub struct RawMap<K, V, H=ABuildHasher> {
    len: AtomicUsize,
    groups: Atomic<Vec<Group<K, V>>>,
    build_hasher: H,
}

impl<K, V> RawMap<K, V, ABuildHasher>
where
    H: BuildHasher,
    K: Hash + PartialEq + Eq + Clone,
{
    pub fn new() -> Self {
        let groups = Atomic::new(iter::repeat_with(|| Group::new()).take(4).collect());

        Self {
            len: AtomicUsize::new(0),
            groups,
            build_hasher: ABuildHasher::new(),
        }
    }

    pub fn get<'guard>(&self, key: &K, guard: &'guard Guard) -> Option<Shared<'guard, V>> {
        let groups = self.groups.load(Ordering::Relaxed, guard);
        let groups = unsafe { groups.as_ref() }.unwrap();

        let mut hasher = self.build_hasher.build_hasher();
        key.hash(&mut hasher);
        let hash: u64 = hasher.finish();

        let group_index = hash as usize % groups.len();
        let group = &groups[group_index];

        let index_in_group = unsafe { Self::heuristic_probe_group(group, hash, true)? };

        let ptr = unsafe {
            (&mut *group.values[index_in_group].get())
                .load(Ordering::Relaxed)
                .as_ref()
                .unwrap()
        };
        Some(ptr)
    }

    #[cfg(target_feature = "sse2")]
    unsafe fn heuristic_probe_group(
        group: &Group<K, V>,
        hash: u64,
        acquire: bool,
    ) -> Option<usize> {
        use std::arch::x86_64::*;

        let control = {
            let a = group.controls[0].load(Ordering::Relaxed);
            let b = group.controls[1].load(Ordering::Relaxed);

            _mm_setr_epi64(b, a)
        };

        let to_find = (0x01 << 7) | (hash >> (64 - 7));
        debug_assert!(to_find <= 0xFF);

        let mask = _mm_cmpeq_epi8(control, _mm_set1_epi8(to_find as i8));

        let mask = _mm_movemask_epi8(mask) as u16;

        // Find set bit in mask.
        let leading_zeroes = mask.leading_zeros();

        if leading_zeroes == 16 {
            None
        } else {
            if acquire {
                unimplemented!("acquire");
            }
            Some(leading_zeroes as usize)
        }
    }
}
