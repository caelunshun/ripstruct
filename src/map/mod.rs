use ahash::ABuildHasher;
use crate::map::raw::RawMap;

mod raw;

pub struct RashMap<K, V, H=ABuildHasher> {
    raw: RawMap<K, V, H>,
}

impl <K, V> RashMap<K, V, ABuildHasher> {
    pub fn new() -> Self {
        Self {
            raw: RawMap::new(),
        }
    }
}

impl <K, V, H> RashMap<K, V, H> {

}
