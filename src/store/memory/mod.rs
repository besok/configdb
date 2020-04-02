//! This module contains memory implementation
//! The general structure is skiplist.
//! For memory checking for not existing entities the cuckoo filter is used
//! For getting a fingerprint from bytes the rabin algorithm is used
pub mod memtable;

use std::path::{PathBuf, Path};
use std::fmt::Error;

type MemResult = Result<(), Error>;


trait MemTable<K: Ord + Clone, V: Clone> {
    fn check(&self, key: K) -> bool;
    fn find(&self, key: K) -> Option<V>;
    fn put(&self, key: K, value: V) -> MemResult;
}

trait Loader<E> {
    fn load_from_disk(path: &Path) -> E;
    fn drop_to_disk(elem: E, path: &Path) -> MemResult;
}