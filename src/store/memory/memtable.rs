use crate::store::structures::skip_list::SkipList;
use crate::store::structures::cuckoo_filter::CuckooFilter;
use std::hash::Hash;
use crate::store::ToBytes;

struct BaseMemTable<K, V>
    where K: Ord + Clone + Hash + ToBytes, V: Clone + ToBytes {
    data: SkipList<K, V>,
    filter: CuckooFilter<K>,
    size: u64,
    limit: u64,
}