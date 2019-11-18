use std::cmp::Ordering;
use crate::store::trees::b_tree::NodeType::{ROOT, LEAF, OTHER};
use crate::store::transaction_log::{ToBytes, FromBytes, LogError};

#[derive(Debug, PartialEq)]
enum NodeType {
    ROOT,
    LEAF,
    OTHER,
}

impl ToBytes for NodeType {
    fn to_bytes(&self) -> Vec<u8> {
        match &self {
            ROOT => vec![1 as u8],
            LEAF => vec![2 as u8],
            _ => vec![3 as u8],
        }
    }
}

impl FromBytes for NodeType {
    fn from_bytes(bytes: &[u8]) -> Result<Self, LogError> {
        return match bytes {
            [b] => match b {
                1 => Ok(ROOT),
                2 => Ok(LEAF),
                _ => Ok(OTHER),
            }
            _ => Err(LogError(String::from("empty bytes or more than one"))),
        };
    }
}

pub trait FixedSized: Sized {
    fn size_in_bytes(&self) -> u32;
}

struct Cell<'a, D>
    where D: PartialOrd + FixedSized
{
    pointer: Option<&'a Node<'a, D>>,
    value: D,
}

impl<'a, D> Cell<'a, D>
    where D: PartialOrd + FixedSized
{
    pub fn new_with_pointer(value: D, pointer: Option<&'a Node<D>>) -> Option<Cell<'a, D>> {
        Some(Cell { value, pointer })
    }
    pub fn new(value: D) -> Option<Cell<'a, D>> {
        Cell::new_with_pointer(value, None)
    }
    pub fn set_value(&mut self, val: D) {
        self.value = val;
    }
    pub fn set_pointer(&mut self, pointer: &'a Node<D>) {
        self.pointer = Some(pointer);
    }
    pub fn compare(&self, next_val: &'a D) -> Option<Ordering> {
        self.value.partial_cmp(next_val)
    }
}

struct Node<'a, D>
    where D: PartialOrd + FixedSized
{
    node_type: NodeType,
    cells: Vec<Cell<'a, D>>,
    next_node: Option<&'a Node<'a, D>>,
}

struct BpTree<'a, D>
    where D: PartialOrd + FixedSized
{
    br_factor: u32,
    root: &'a Node<'a, D>,
}

#[cfg(test)]
mod tests {
    use crate::store::trees::b_tree::{NodeType, Cell, FixedSized};
    use crate::store::trees::b_tree::NodeType::{ROOT, LEAF, OTHER};
    use crate::store::transaction_log::{ToBytes, LogError, FromBytes};
    use std::cmp::Ordering;

    #[test]
    fn node_type_test() {
        assert_eq!(ROOT.to_bytes(), vec![1]);
        assert_eq!(LEAF.to_bytes(), vec![2]);
        assert_eq!(OTHER.to_bytes(), vec![3]);

        if let Ok(nt) = NodeType::from_bytes(vec![1].as_slice()) {
            assert_eq!(nt, ROOT)
        };
        if let Ok(nt) = NodeType::from_bytes(vec![2].as_slice()) {
            assert_eq!(nt, LEAF)
        };
        if let Ok(nt) = NodeType::from_bytes(vec![3].as_slice()) {
            assert_eq!(nt, OTHER)
        };
        if let Ok(nt) = NodeType::from_bytes(vec![4].as_slice()) {
            assert_eq!(nt, OTHER)
        };
        if let Err(le) = NodeType::from_bytes(vec![].as_slice()) {
            assert_eq!(le.0, String::from("empty bytes or more than one"))
        };
        if let Err(le) = NodeType::from_bytes(vec![1, 2].as_slice()) {
            assert_eq!(le.0, String::from("empty bytes or more than one"))
        };
    }

    #[test]
    fn simple_cell_test() {
        impl FixedSized for i32 {
            fn size_in_bytes(&self) -> u32 {
                4
            }
        }

        if let Some(left) = Cell::new(10) {
            assert_eq!(left.value, 10);
            if let Some(right) = Cell::new(10) {
                if let Some(eq) = (&left).compare(&right.value) {
                    assert_eq!(eq,Ordering::Equal)
                } else {
                    panic!(" should be eq")
                }
            }
            if let Some(right) = Cell::new(11) {
                if let Some(ls) = (&left).compare(&right.value) {
                    assert_eq!(ls,Ordering::Less)
                } else {
                    panic!(" should be less")
                }
            }
            if let Some(right) = Cell::new(9) {
                if let Some(gr) = (&left).compare(&right.value) {
                    assert_eq!(gr,Ordering::Greater)
                } else {
                    panic!(" should be greater")
                }
            }
        }
    }
}