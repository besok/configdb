use std::marker::PhantomData;
use std::hash::{Hash, Hasher};
use crate::store::memory::fingerprint::{Fingerprint, RabinFingerprint, Polynomial};
use crate::store::transaction_log::ToBytes;
use std::collections::hash_map::DefaultHasher;
use rand::Rng;

struct Bucket {
    base: Vec<Option<i64>>,
    idx: usize,
}

#[derive(Debug)]
enum InsertResult {
    Done(usize),
    Full,
    Fail(String),
}


impl Bucket {
    fn new() -> Self {
        Bucket {
            base: vec![None; 8],
            idx: 0,
        }
    }
    fn new_with(val:i64) -> Self{
        let mut bucket = Bucket::new();
        bucket.insert(val);
        bucket
    }

    fn insert(&mut self, v: i64) {
        self.base.insert(self.idx, Some(v));
        self.idx += 1
    }

    fn swap(&mut self, v: i64) -> Option<i64> {
        let mut rng = rand::thread_rng();
        let idx_swap = rng.gen_range(0, self.idx);
        let old_val = self.base.get(idx_swap).and_then(|v| v.clone());
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

    fn swap(&mut self, idx: usize, v: i64) -> Option<i64> {
        self.delegate.get_mut(idx).and_then(|b| b.swap(v))
    }

    fn insert(&mut self, idx: usize, v: i64) -> InsertResult {
        let len = self.len();
        if len <= idx{
            return InsertResult::Fail(String::from(format!("idx {} > len {}", idx, len)))
        }

        match self.delegate.get_mut(idx) {
            Some(b) if b.is_full() => InsertResult::Full,
            Some(b) => {
                b.insert(v);
                InsertResult::Done(idx)
            }
            None => {
                self.delegate.insert(idx,Bucket::new_with(v));
                InsertResult::Done(idx)
            },
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
        CuckooFilter::new(2 << 16, 0.8)
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

        let hash_num = self.find_bucket_number(hash);

        match self.table.insert(hash_num, fpr) {
            InsertResult::Full => {
                let fpr_num = self.find_bucket_number(hash_num as i64 ^ fpr);
                match self.table.insert(fpr_num, fpr) {
                    InsertResult::Full => {
                        let mut idx = 0;
                        let mut num = if bool_rand() { hash_num } else { fpr_num };
                        let mut v = fpr;

                        while idx < 1024 {
                            match self.table.swap(num, v) {
                                None => return InsertResult::Fail(String::from("the value not found")),
                                Some(next_v) => {
                                    let next_num = self.find_bucket_number(next_v ^ num as i64);
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

    fn contains(&mut self, val: &T) -> bool {
        let fpr: i64 = self.fpr.calculate(val.to_bytes()).unwrap();
        let hash = find_hash(val);

        let idx = self.find_bucket_number(hash);
        if self.table.contains(idx, fpr) {
            return true;
        }
        let idx = self.find_bucket_number(idx as i64 ^ fpr);
        if self.table.contains(idx, fpr) {
            return true;
        }

        false
    }
    fn find_bucket_number(&self, hash: i64) -> usize {
        (hash & (self.table.len() -1 ) as i64) as usize
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
    use crate::store::memory::cuckoo_filter::{CuckooFilter, Bucket, find_hash, InsertResult};
    use crate::store::transaction_log::ToBytes;

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
    fn cuckoo_test() {
        let mut f: CuckooFilter<i32> = CuckooFilter::new(2 << 16, 0.8);


        for el in 1..10000 {
            println!(" el {}",el);
            match f.insert(&el){
                InsertResult::Done(_) => assert_eq!(f.contains(&el), true),
                r @ _  => panic!("{:?} ",r),
            }
        }
        assert_eq!(false, f.contains(&10001))
    }

    #[test]
    fn hash_test() {
        let mut t: CuckooFilter<i64> = CuckooFilter::default();
        let fpr = 123;
        let hash = find_hash(&567);
        let i1 = t.find_bucket_number(hash);
        let i2 = t.find_bucket_number((fpr ^ i1) as i64);
        let i3 = t.find_bucket_number((fpr ^ i2) as i64);

        assert_eq!(i1, i3)
    }
}

