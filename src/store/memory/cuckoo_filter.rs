use std::marker::PhantomData;
use std::hash::{Hash, Hasher};
use crate::store::memory::fingerprint::{Fingerprint, RabinFingerprint, Polynomial};
use crate::store::transaction_log::ToBytes;
use std::collections::hash_map::DefaultHasher;

struct Bucket {
    base: Vec<Option<i64>>,
    idx: usize,
}

enum InsertResult {
    Done,
    Full,
    Fail,
}

impl Bucket {
    fn new() -> Self {
        Bucket {
            base: vec![None; 8],
            idx: 0,
        }
    }

    fn insert(&mut self, v: i64) {
        self.base.insert(self.idx, Some(v));
        self.idx += 1
    }

    fn contains(&self, fp: i64) -> bool {
        self.base.contains(&Some(fp))
    }

    fn is_empty(&self) -> bool {
        self.idx == 0
    }
    fn is_full(&self) -> bool {
        self.idx == 8
    }
}

impl Clone for Bucket {
    fn clone(&self) -> Self {
        Bucket {
            base: self.base.clone(),
            idx: self.idx.clone(),
        }
    }
}

struct Table {
    delegate: Vec<Bucket>
}

impl Table {
    fn new(cap: usize) -> Self {
        Table {
            delegate: vec![Bucket::new(); cap]
        }
    }
    fn len(&self) -> usize {
        self.delegate.len()
    }
    fn contains(&self, idx: usize, v: i64) -> bool {
        match self.delegate.get(idx) {
            Some(b) => b.contains(v),
            None => false,
        }
    }
    fn insert(&mut self, idx: usize, v: i64) -> InsertResult {
        match self.delegate.get_mut(idx) {
            Some(ref b) if b.is_full() => InsertResult::Full,
            Some(b) => {
                b.insert(v);
                InsertResult::Done
            }
            None => InsertResult::Fail,
        }
    }
}

struct CuckooFilter<T: Hash + ToBytes> {
    size: usize,
    table: Table,
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
            table: Table::new(cap),
            size: 0,
            load_factor: lf,
            fpr: RabinFingerprint::new_default(),
            _mark: PhantomData,
        }
    }


    fn insert(&mut self, v: &T) -> InsertResult {
        let fpr: i64 = self.fpr.calculate(v.to_bytes()).unwrap();
        let hash = find_hash(v);
        let hash_fpr = hash ^ fpr;

        let n = find_bucket_number(self.table.len(), hash);
        match self.table.insert(n, fpr) {
            InsertResult::Full => {
                let n = find_bucket_number(self.table.len(), hash_fpr);
                match self.table.insert(n, fpr) {
                    InsertResult::Full => {
                        InsertResult::Full
                    }
                    r @ _ => r
                }
            }
            r @ _ => r
        }
    }

    fn contains(&mut self, val: &T) -> bool {
        let fpr: i64 = self.fpr.calculate(val.to_bytes()).unwrap();
        let hash = find_hash(val);

        let idx = find_bucket_number(self.table.len(), hash);
        if self.table.contains(idx, fpr) {
            return true;
        }
        let idx = find_bucket_number(self.table.len(), idx as i64 ^ fpr);
        if self.table.contains(idx, fpr) {
            return true;
        }

        false
    }
}

fn find_hash<T: Hash>(entity: &T) -> i64 {
    let mut s = DefaultHasher::new();
    entity.hash(&mut s);
    s.finish() as i64
}

fn find_bucket_number(size: usize, hash: i64) -> usize {
    (hash & size as i64) as usize
}

#[cfg(test)]
mod tests {
    use crate::store::memory::cuckoo_filter::{CuckooFilter, Bucket, find_bucket_number, find_hash};
    use crate::store::transaction_log::ToBytes;

    impl ToBytes for i64 {
        fn to_bytes(&self) -> Vec<u8> {
            self.to_be_bytes().to_vec()
        }
    }


    #[test]
    fn bucket_test() {
        let mut bucket = Bucket::new();
        assert_eq!(false, bucket.contains(1));
        assert_eq!(false, bucket.is_full());
        assert_eq!(true, bucket.is_empty());

        bucket.insert(1);
        assert_eq!(true, bucket.contains(1));
        assert_eq!(false, bucket.is_full());
        assert_eq!(false, bucket.is_empty());

        for el in 2..9 {
            bucket.insert(el);
        }

        assert_eq!(true, bucket.is_full());
        assert_eq!(false, bucket.is_empty());
    }

    #[test]
    fn hash_test() {
        let fpr = 123;
        let hash = find_hash(&567);
        let i1 = find_bucket_number(1000, hash);
        let i2 = find_bucket_number(1000, (fpr ^ i1) as i64);
        let i3 = find_bucket_number(1000, (fpr ^ i2) as i64);

        assert_eq!(i1,i3)
    }
}

