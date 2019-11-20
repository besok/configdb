use std::cmp::Ordering;

pub trait FixedSized: Sized {
    fn size_in_bytes(&self) -> u32;
}

#[derive(Debug)]
pub enum Elem<'a, K, P>
    where K: PartialOrd,
{
    Cell {
        pnt: Option<&'a Elem<'a, K, P>>,
        key: &'a K,
    },

    EndCell {
        pnt: &'a P,
        key: &'a K,
    },

    Node {
        cells: Vec<&'a Elem<'a, K, P>>,
        next_node: Option<&'a Elem<'a, K, P>>,
    },

    Tree {
        br_factor: u32,
        root: &'a Elem<'a, K, P>,
    },
    Empty,
}

impl<'a, K, P> Elem<'a, K, P>
    where K: PartialOrd,
{
    pub fn new_cell(key: &'a K,
                    pnt: Option<&'a Elem<'a, K, P>>) -> Elem<'a, K, P> {
        Elem::Cell { key, pnt }
    }
    pub fn new_end_cell(key: &'a K,
                        pnt: &'a P) -> Elem<'a, K, P> {
        Elem::EndCell { key, pnt }
    }
    pub fn new_node(cells: Vec<&'a Elem<'a, K, P>>,
                    next_node: Option<&'a Elem<'a, K, P>>) -> Elem<'a, K, P> {
        Elem::Node { cells, next_node }
    }
    pub fn new_tree(br_factor: u32,
                    root: &'a Elem<'a, K, P>) -> Elem<'a, K, P> {
        match root {
            Elem::Node { .. } => Elem::Tree { br_factor, root },
            _ => Elem::Empty,
        }
    }
}

impl<'a, K, P> PartialOrd for Elem<'a, K, P>
    where K: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let cmp_keys = |key| match self {
            Elem::Cell { key: src, .. } |
            Elem::EndCell { key: src, .. } => src.partial_cmp(key),
            _ => None,
        };
        match other {
            Elem::Cell { key, .. } |
            Elem::EndCell { key, .. } => cmp_keys(key),
            _ => None,
        }
    }
}

impl<'a, K, P> PartialEq for Elem<'a, K, P>
    where K: PartialOrd,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Elem::Tree { root: src, .. }, Elem::Tree { root: trg, .. }) => src == trg,
            (Elem::Node { cells: src, .. }, Elem::Node { cells: trg, .. }) => src == trg,
            (Elem::Cell { key: src, .. }, Elem::Cell { key: trg, .. }) => src == trg,
            (Elem::EndCell { key: src, .. }, Elem::EndCell { key: trg, .. }) => src == trg,
            _ => false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;
    use crate::store::trees::b_tree::Elem;

    #[test]
    fn simple_cell_test() {
        let cell_one = Elem::<_, i8>::new_cell(&10, None);
        let cell_two = Elem::new_cell(&10, None);
        if let Some(Ordering::Equal) = cell_one.partial_cmp(&cell_two) {} else {
            panic!("should be eq")
        }
        let cell_two = Elem::new_cell(&11, None);
        if let Some(Ordering::Less) = cell_one.partial_cmp(&cell_two) {} else {
            panic!("should be ls")
        }
        let cell_two = Elem::new_cell(&9, None);
        if let Some(Ordering::Greater) = cell_one.partial_cmp(&cell_two) {} else {
            panic!("should be gr")
        }
        let cell_two = Elem::new_node(vec![], None);
        if let Some(_) = cell_one.partial_cmp(&cell_two) {
            panic!("should be none")
        }
    }

    #[test]
    fn simple_tree_test() {
        let cell_end = Elem::<_, i8>::new_end_cell(&10, &1);
        let leaf_node = Elem::new_node(vec![&cell_end],None);
        let cell = Elem::<_, i8>::new_cell(&10, Some(&leaf_node));

        let root_node = Elem::new_node(vec![&cell],None);

        let tree = Elem::new_tree(3, &root_node);
        assert_eq!(
            "Tree { br_factor: 3, root: Node \
            { cells: [Cell { pnt: Some(Node \
            { cells: [EndCell { pnt: 1, key: 10 }], \
            next_node: None }), key: 10 }], next_node: None } }"
                   ,format!("{:?}", tree))
    }
}