use std::marker::PhantomData;
use std::hash::{Hash, Hasher};
use crate::store::memory::fingerprint::{Fingerprint, RabinFingerprint, Polynomial};
use crate::store::transaction_log::ToBytes;
use std::collections::hash_map::DefaultHasher;

enum InsertResult {
}

struct CuckooFilter<T: Hash + ToBytes> {
    size: usize,
    table: Vec<Vec<bool>>,
    fpr: RabinFingerprint,
    load_factor: f32,
    _mark: PhantomData<T>,
}

impl<T: Hash + ToBytes> CuckooFilter<T> {
    fn default() -> Self {
        CuckooFilter::new(2 >> 16, 0.8)
    }
    fn new(cap: usize, lf: f32) -> Self {
        CuckooFilter {
            table: vec![vec![false; 8]; cap],
            size: 0,
            load_factor: lf,
            fpr: RabinFingerprint::new_default(),
            _mark: PhantomData,
        }
    }

    fn is_full(&self) -> bool {
        let x = (self.table.len() * 8) as f32;
        self.size as f32 / x > self.load_factor
    }
    fn insert(&mut self, entity: &T) -> InsertResult {
        let len = self.table.len();
        let bucket = find_bucket(len, find_hash(entity));


    }

    fn contains(&mut self, entity: &T) -> bool {
        let len = self.table.len();

        let bucket = find_bucket(len, find_hash(entity));
        if let Some(v) = self.table.get(bucket) {
            if !v.is_empty() {
                return true;
            }
        }

        self.fpr.calculate(entity.to_bytes())
            .map(|v| find_bucket(len, v))
            .and_then(|v| self.table.get(v))
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }
}

fn find_hash<T: Hash>(entity: &T) -> i64 {
    let mut s = DefaultHasher::new();
    entity.hash(&mut s);
    s.finish() as i64
}

fn find_bucket(size: usize, hash: i64) -> usize {
    (hash & size as i64) as usize
}

#[cfg(test)]
mod tests {
    use crate::store::memory::cuckoo_filter::CuckooFilter;
    use crate::store::transaction_log::ToBytes;

    impl ToBytes for i64 {
        fn to_bytes(&self) -> Vec<u8> {
            self.to_be_bytes().to_vec()
        }
    }


    #[test]
    fn test() {
        let mut filter: CuckooFilter<i64> = CuckooFilter::default();
        assert_eq!(filter.contains(&10_i64), false);
    }
}

