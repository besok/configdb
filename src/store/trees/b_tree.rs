use std::cmp::Ordering;
use crate::store::transaction_log::{ToBytes, FromBytes, LogError};

pub trait FixedSized: Sized {
    fn size_in_bytes(&self) -> u32;
}


enum Elem<'a, D, P>
    where D: PartialOrd,
{
    Cell {
        pointer: Option<&'a Elem<'a, D, P>>,
        value: D,
    },

    EndCell {
        pointer: P,
        value: D,
    },

    Node {
        cells: Vec<Elem<'a, D, P>>,
        next_node: Option<&'a Elem<'a, D, P>>,
    },

    Tree {
        br_factor: u32,
        root: &'a Elem<'a, D, P>,
    },
}


#[cfg(test)]
mod tests {
    use crate::store::trees::b_tree::{Cell, FixedSized};
    use crate::store::transaction_log::{ToBytes, LogError, FromBytes};
    use std::cmp::Ordering;

    #[test]
    fn simple_cell_test() {}
}