use std::cmp::Ordering;
use crate::store::transaction_log::{ToBytes, FromBytes, LogError};

pub trait FixedSized: Sized {
    fn size_in_bytes(&self) -> u32;
}


enum Elem<'a, K, P>
    where K: PartialOrd,
{
    Cell {
        pointer: Option<&'a Elem<'a, K, P>>,
        key: K,
    },

    EndCell {
        pointer: P,
        key: K,
    },

    Node {
        cells: Vec<Elem<'a, K, P>>,
        next_node: Option<&'a Elem<'a, K, P>>,
    },

    Tree {
        br_factor: u32,
        root: &'a Elem<'a, K, P>,
    },
}

impl<'a, K, P> Elem<'a, K, P>
    where K: PartialOrd, {
    pub fn new_cell(key: K, pointer: Option<&'a Elem<'a, K, P>>) -> Elem<'a, K, P> {
        return Elem::Cell { key, pointer };
    }
    pub fn new_end_cell(key: K, pointer: P) -> Elem<'a, K, P> {
        Elem::EndCell { key, pointer }
    }
    pub fn new_node(cells: Vec<Elem<'a, K, P>>,
                    next_node: Option<&'a Elem<'a, K, P>>) -> Elem<'a, K, P> {
        Elem::Node { cells, next_node }
    }
}


#[cfg(test)]
mod tests {
    use crate::store::transaction_log::{ToBytes, LogError, FromBytes};
    use std::cmp::Ordering;

    #[test]
    fn simple_cell_test() {}
}