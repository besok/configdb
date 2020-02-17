use crate::store::memory::cuckoo_filter::{CuckooFilter, InsertResult};
use std::hash::Hash;
use crate::store::transaction_log::ToBytes;


struct Filter<T: Hash + ToBytes> {
    delegate: CuckooFilter<T>,
    index: usize,
}

impl<T> Filter<T> where T: Hash + ToBytes {
    fn new(index: usize, cap: usize) -> Self {
        Filter { index, delegate: CuckooFilter::new(cap, 0.8) }
    }
    fn default(index: usize) -> Self {
        Filter { index, delegate: CuckooFilter::default() }
    }
    fn put(&mut self, k: &T) -> InsertResult {
        self.delegate.insert(k)
    }
    fn contains(&mut self, k: &T) -> Option<usize> {
        if self.delegate.contains(k) {
            Some(self.index)
        } else {
            None
        }
    }
}

pub struct FilterHandler<T: Hash + ToBytes> {
    filters: Vec<Filter<T>>
}


impl<T> FilterHandler<T> where T: Hash + ToBytes {
    pub fn new() -> Self {
        FilterHandler {
            filters: vec![]
        }
    }
    pub fn init_filter(&mut self, index: usize, cap: usize) {
        self.filters.insert(index, Filter::new(index, cap))
    }
    pub fn add_to_filter(&mut self, index: usize, key: &T) -> InsertResult {
        match self.filters.get_mut(index) {
            Some(f) => {
                match f.put(key) {
                    r @ InsertResult::Done(_) |
                    r @ InsertResult::Fail(_) => r,
                    InsertResult::Full => {
                        let new_cap = f.delegate.cap() * 2;
                        let new_filter = Filter::new(index, new_cap);
                        self.filters.insert(index, new_filter);

                        self.add_to_filter(index, key)
                    }
                }
            }
            None => InsertResult::Fail(String::from("the filter with index does not exist"))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_test() {}
}