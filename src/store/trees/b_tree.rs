use std::cmp::Ordering;
use std::fmt::Debug;
use std::cell::{RefCell, Cell, Ref};
use std::borrow::Borrow;
use std::rc::Rc;

enum SearchRes {
    Down(usize),
    Found(usize),
    None,
}

#[derive(Debug)]
enum InsertRes
{
    None,
}

#[derive(Debug)]
enum Node<K, P>
    where K: PartialOrd + Debug
{
    Node {
        keys: Vec<K>,
        links: Vec<Rc<Node<K, P>>>,
    },
    Leaf {
        keys: Vec<K>,
        pts: Vec<Rc<P>>,
    },
}

impl<K, P> Node<K, P>
    where K: PartialOrd + Debug
{
    pub fn new_node(keys: Vec<K>, links: Vec<Node<K, P>>) -> Node<K, P> {
        Node::Node { keys, links: links.into_iter().map(|x| Rc::new(x)).collect() }
    }
    pub fn new_leaf(keys: Vec<K>, pts: Vec<P>) -> Node<K, P> {
        Node::Leaf { keys, pts: pts.into_iter().map(|x| Rc::new(x)).collect() }
    }

    fn get_node(&self, i: usize) -> Option<Rc<Node<K, P>>> {
        match self {
            Node::Node { links, .. } =>
                links.get(i).map(|v| v.clone()),
            Node::Leaf { .. } => None,
        }
    }

    fn get_ptr(&self, i: usize) -> Option<Rc<P>> {
        match self {
            Node::Node { .. } => None,
            Node::Leaf { pts, .. } => pts.get(i).map(|v| v.clone()),
        }
    }
    fn search(&self, key: &K) -> SearchRes {
        match self {
            Node::Node { keys, .. } => {
                let mut last: usize = 0;
                for (i, k) in keys.iter().enumerate() {
                    match key.partial_cmp(k) {
                        Some(Ordering::Equal) |
                        Some(Ordering::Less) => return SearchRes::Down(i),
                        _ => last = i
                    }
                }
                return SearchRes::Down(last);
            }
            Node::Leaf { keys, .. } =>
                for (i, k) in keys.iter().enumerate() {
                    match key.partial_cmp(k) {
                        Some(Ordering::Equal) => return SearchRes::Found(i),
                        Some(Ordering::Less) => break,
                        _ => {}
                    }
                },
        }
        return SearchRes::None;
    }
}


struct Tree<K, P>
    where K: PartialOrd + Debug
{
    diam: usize,
    root: Rc<Node<K, P>>,
}

impl<K, P> Tree<K, P>
    where K: PartialOrd + Debug
{
    fn search(&self, key: &K) -> Option<Rc<P>> {
        let mut node = self.root.clone();

        loop {
            match node.search(key) {
                SearchRes::Down(i) =>
                    if let Some(nd) = node.get_node(i) {
                        node = nd
                    } else { return None; },
                SearchRes::Found(i) => return node.get_ptr(i),
                SearchRes::None => return None,
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::store::trees::b_tree::Node;
    use crate::store::trees::b_tree::Tree;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::borrow::Borrow;

    #[test]
    fn simple_test() {
        let leaf = Node::new_leaf(vec![1],vec![10]);
        let node = Node::new_node(vec![2],vec![leaf]);

        let tree = Tree { diam: 0, root: Rc::new(node) };


        if let Some(v) = (&tree).search(&1) {
            assert_eq!(v, Rc::new(10))
        } else {
            panic!("")
        }
        if let Some(_) = (&tree).search(&3) {
            panic!("")
        }
    }
}