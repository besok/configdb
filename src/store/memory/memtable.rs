use crate::store::memory::skip_list::SkipList;

struct Memtable<K: Ord + Clone, V: Clone> {
    data: SkipList<K, V>
}