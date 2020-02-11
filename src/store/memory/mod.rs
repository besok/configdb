//! This module contains memory implementation
//! The general structure is skiplist.
//! For memory checking for not existing entities the cuckoo filter is used
//! For getting a fingerprint from bytes the rabin algorithm is used

pub mod skip_list;
pub mod cuckoo_filter;
pub mod fingerprint;
pub mod memtable;