//! A cuckoo filter is a compact variant of a cuckoo hash
//! table [21] that stores only fingerprints—a bit string derived
//! from the item using a hash function—for each item inserted,
//! instead of key-value pairs. The filter is densely filled with
//! fingerprints (e.g., 95% entries occupied), which confers high
//! space efficiency. A set membership query for item x simply
//! searches the hash table for the fingerprint of x, and returns
//! true if an identical fingerprint is found.
//! # Examples
//! ```
//!        let mut t: CuckooFilter<i64> = CuckooFilter::default();
//!        match f.insert(&1) {
//!                InsertResult::Done(_) => (),
//!                InsertResult::Fail(exp) => (),
//!                InsertResult::Full => (),
//!            }
//!         assert_eq!(f.contains(&1), true);
//!         assert_eq!(f.contains(&10), false);
//! ```
//!
//!
//!
use std::marker::PhantomData;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use rand::Rng;
use crate::store::structures::fingerprint::{RabinFingerprint, Fingerprint};
use crate::store::ToBytes;

struct Bucket {
    base: Vec<Option<i64>>,
    idx: usize,
    cap: usize,
}

#[derive(Debug)]
pub enum InsertResult {
    Done(usize),
    Full,
    Fail(String),
}


impl Bucket {
    fn new(cap: usize) -> Self {
        Bucket {
            base: vec![None; cap],
            idx: 0,
            cap,
        }
    }

    fn new_with(val: i64, cap: usize) -> Self {
        let mut bucket = Bucket::new(cap);
        bucket.insert(val);
        bucket
    }

    fn insert(&mut self, v: i64) {
        if self.contains(v) {
            return;
        }

        self.base.insert(self.idx, Some(v));
        self.idx += 1
    }

    fn swap(&mut self, v: i64) -> Option<i64> {
        let mut rng = rand::thread_rng();
        let idx_swap = rng.gen_range(0, self.idx);
        let old_val =
            self.base
                .get(idx_swap)
                .and_then(|v| v.clone());
        self.base.insert(idx_swap, Some(v));
        old_val
    }

    fn contains(&self, fp: i64) -> bool {
        self.base.contains(&Some(fp))
    }

    fn is_empty(&self) -> bool {
        self.idx == 0
    }
    fn is_full(&self) -> bool {
        self.idx == self.cap
    }
}


impl Clone for Bucket {
    fn clone(&self) -> Self {
        Bucket {
            base: self.base.clone(),
            idx: self.idx.clone(),
            cap: self.cap.clone(),
        }
    }
}

struct Table {
    delegate: Vec<Bucket>,
    bucket_cap: usize,
}

impl Table {
    fn new(cap: usize, bucket_cap: usize) -> Self {
        Table {
            delegate: vec![Bucket::new(bucket_cap); cap],
            bucket_cap,
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

    fn swap_rand(&mut self, idx: usize, v: i64) -> Option<i64> {
        self.delegate
            .get_mut(idx)
            .and_then(|b| b.swap(v))
    }

    fn insert(&mut self, idx: usize, v: i64) -> InsertResult {
        let len = self.len();
        if len <= idx {
            return InsertResult::Fail(String::from(format!("idx {} > len {}", idx, len)));
        }

        match self.delegate.get_mut(idx) {
            Some(b) if b.is_full() => InsertResult::Full,
            Some(b) => InsertResult::Done({
                b.insert(v);
                idx
            }),
            None => InsertResult::Done({
                self.delegate.insert(idx, Bucket::new_with(v, self.bucket_cap));
                idx
            })
        }
    }
}

pub struct CuckooFilter<T: Hash + ToBytes> {
    table: Table,
    fpr: RabinFingerprint,
    load_factor: f32,
    _mark: PhantomData<T>,
}

impl<T: Hash + ToBytes> CuckooFilter<T> {
    pub fn default() -> Self {
        CuckooFilter::new(2 << 16, 0.8)
    }
    pub fn new_with(cap: usize, lf: f32, bucket_cap: usize) -> Self {
        CuckooFilter {
            table: Table::new(cap, bucket_cap),
            load_factor: lf,
            fpr: RabinFingerprint::default(),
            _mark: PhantomData,
        }
    }
    pub fn new(cap: usize, lf: f32) -> Self {
        CuckooFilter {
            table: Table::new(cap, 8),
            load_factor: lf,
            fpr: RabinFingerprint::default(),
            _mark: PhantomData,
        }
    }

    pub fn insert(&mut self, v: &T) -> InsertResult {
        let fpr: i64 = self.fpr.calculate(v.to_bytes())
            .expect("impossible to calculate the polynomial.");
        let hash = find_hash(v);

        let bucket = self.bucket(hash);

        match self.table.insert(bucket, fpr) {
            InsertResult::Full => {
                let fpr_num = self.bucket(bucket as i64 ^ fpr);
                match self.table.insert(fpr_num, fpr) {
                    InsertResult::Full => {
                        let mut idx = 0;
                        let mut num = if bool_rand() { bucket } else { fpr_num };
                        let mut v = fpr;

                        while idx < 512 {
                            match self.table.swap_rand(num, v) {
                                None => return InsertResult::Fail(String::from("the value not found")),
                                Some(next_v) => {
                                    let next_num = self.bucket(next_v ^ num as i64);
                                    match self.table.insert(next_num, v) {
                                        InsertResult::Full => {
                                            idx += 1;
                                            v = next_v;
                                            num = next_num;
                                        }
                                        r @ _ => return r,
                                    }
                                }
                            }
                        }
                        InsertResult::Full
                    }
                    r @ _ => r
                }
            }
            r @ _ => r
        }
    }
    pub fn cap(&self) -> usize {
        self.table.len() * self.table.bucket_cap
    }
    pub fn contains(&mut self, val: &T) -> bool {
        let fpr: i64 = self.fpr.calculate(val.to_bytes())
            .expect("impossible to calculate the polynomial.");
        let hash = find_hash(val);

        let idx = self.bucket(hash);
        if self.table.contains(idx, fpr) {
            return true;
        }
        let idx = self.bucket(idx as i64 ^ fpr);
        if self.table.contains(idx, fpr) {
            return true;
        }

        false
    }
    fn bucket(&self, hash: i64) -> usize {
        (hash & (self.table.len() - 1) as i64) as usize
    }
}

fn bool_rand() -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_bool(0.5)
}

fn find_hash<T: Hash>(entity: &T) -> i64 {
    let mut s = DefaultHasher::new();
    entity.hash(&mut s);
    s.finish() as i64
}

#[cfg(test)]
mod tests {
    use crate::store::structures::cuckoo_filter::{Bucket, CuckooFilter, InsertResult, find_hash};
    use crate::store::ToBytes;

    impl ToBytes for i64 {
        fn to_bytes(&self) -> Vec<u8> {
            self.to_be_bytes().to_vec()
        }
    }

    impl ToBytes for i32 {
        fn to_bytes(&self) -> Vec<u8> {
            self.to_be_bytes().to_vec()
        }
    }


    #[test]
    fn bucket_test() {
        let mut bucket = Bucket::new(8);
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
    fn full_cuckoo_test() {
        let mut f: CuckooFilter<i32> = CuckooFilter::new_with(1, 0.8, 1);
        f.insert(&1);
        if let InsertResult::Full = f.insert(&1) {} else {
            assert!(false);
        };
    }

    #[test]
    fn cuckoo_test() {
        let mut f: CuckooFilter<i32> = CuckooFilter::new(2 << 16, 0.8);


        for el in 1..10000 {
            match f.insert(&el) {
                InsertResult::Done(_) => assert_eq!(f.contains(&el), true),
                r @ _ => panic!("{:?} ", r),
            }
        }
        assert_eq!(false, f.contains(&10001))
    }

    #[test]
    fn hash_test() {
        let mut t: CuckooFilter<i64> = CuckooFilter::default();
        let fpr = 123;
        let hash = find_hash(&567);
        let i1 = t.bucket(hash);
        let i2 = t.bucket((fpr ^ i1) as i64);
        let i3 = t.bucket((fpr ^ i2) as i64);

        assert_eq!(i1, i3)
    }
}

