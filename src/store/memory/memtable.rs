use crate::store::memory::skip_list::SkipList;
use std::hash::Hash;
use crate::store::transaction_log::{ToBytes, StoreResult};

struct MemTable<K, V>
    where K: Ord + Clone + Hash + ToBytes,
          V: Clone + ToBytes {
    data: SkipList<K, MemValue<V>>,
    limit_items: usize,
    limit_bytes: usize,
    size_bytes: usize,
}

struct MemValue<V: Clone + ToBytes> {
    value: V,
    update_time: u128,
}

impl<V: Clone + ToBytes> ToBytes for MemValue<V> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend_from_slice(&self.value.to_bytes());
        bytes.extend_from_slice(&self.update_time.to_be_bytes());
        bytes
    }
}

impl<V: Clone + ToBytes> Clone for MemValue<V> {
    fn clone(&self) -> Self {
        MemValue {
            value: self.value.clone(),
            update_time: self.update_time.clone(),
        }
    }
}

impl<K, V> MemTable<K, V> where K: Ord + Clone + Hash + ToBytes,
                                V: Clone + ToBytes {
    pub fn default() -> Self {
        MemTable {
            data: SkipList::new(),
            size_bytes: 0,
            limit_items: 2 << 8,
            limit_bytes: 2 << 13,
        }
    }

    pub fn new(exp_capacity: usize, limit_items: usize, limit_bytes: usize) -> Self {
        MemTable {
            data: SkipList::with_capacity(exp_capacity),
            limit_items,
            limit_bytes,
            size_bytes: 0,
        }
    }

    pub fn add(&mut self, key: K, value: V) -> StoreResult<MemTableResult> {
//        if let Some(v) = self.data.insert(key, value) {
//            self.size_bytes -= v.to_bytes().len();
//            self.size_bytes -= key.to_bytes().len();
//        }
//
//        self.size_bytes += key.to_bytes().len();
//        self.size_bytes += value.to_bytes().len();
//
//        if self.is_limit() {
//            return self.drop_to_store();
//        }

        Ok(MemTableResult::Ok)
    }

    pub fn find(&self, key:&K) -> Option<V>{
        None
    }
    pub fn remove(&mut self, key:&K) -> StoreResult<MemTableResult> {
        Ok(MemTableResult::Ok)
    }

    fn is_limit(&self) -> bool {
        self.size_bytes >= self.limit_bytes || self.data.size() >= self.limit_items
    }

    fn drop_to_store(&mut self) -> StoreResult<MemTableResult> {
        self.data.clear();
        self.size_bytes = 0;
        Ok(MemTableResult::Full)
    }
}

enum MemTableResult {
    Full,
    Ok,
    Fail,
}

