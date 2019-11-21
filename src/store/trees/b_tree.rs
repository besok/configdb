use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem;
use std::ptr::replace;

enum Res {
    Down(usize),
    Found(usize),
    None,
}

enum Node<'a, K, P>
    where K: PartialOrd + Debug
{
    Node {
        keys: Vec<K>,
        links: Vec<Node<'a, K, P>>,
    },
    Leaf {
        keys: Vec<K>,
        pts: Vec<P>,
        link: Option<&'a Node<'a, K, P>>,
    },
}

impl<'a, K, P> Node<'a, K, P>
    where K: PartialOrd + Debug
{
    fn key_len(&self) -> usize {
        match self {
            Node::Node { keys, .. } |
            Node::Leaf { keys, .. } => keys.len(),
        }
    }

    fn get_node(&'a self, i: usize) -> Option<&'a Node<'a, K, P>> {
        match &self {
            Node::Node { links, .. } => links.get(i),
            _ => None
        }
    }
    fn get_pointer(&'a self, i: usize) -> Option<&'a P> {
        match &self {
            Node::Leaf { pts, .. } => pts.get(i),
            _ => None
        }
    }
    fn insert_to_leaf(&mut self, key: K, p: P) -> Result<(), String> {
        match self {
            Node::Node { .. } => Err(String::from("only for leafs")),
            Node::Leaf { keys, pts, .. } => {
                for (i, el) in keys.iter().enumerate() {
                    match key.partial_cmp(el) {
                        Some(Ordering::Less) => {
                            keys.insert(i, key);
                            pts.insert(i, p);
                        }
                        Some(Ordering::Greater) => {}
                        Some(Ordering::Equal) => {!},
                        None => {}
                    }
                }
            }
        }
    }
    fn search(&self, key: &K) -> Res {
        match self {
            Node::Node { keys, .. } => {
                for (i, k) in keys.iter().enumerate() {
                    match key.partial_cmp(k) {
                        Some(Ordering::Equal) |
                        Some(Ordering::Less) => {
                            return Res::Down(i);
                        }
                        _ => {}
                    }
                }
                return Res::Down(self.key_len());
            }
            Node::Leaf { keys, .. } =>
                for (i, k) in keys.iter().enumerate() {
                    match key.partial_cmp(k) {
                        Some(Ordering::Equal) => return Res::Found(i),
                        Some(Ordering::Less) => break,
                        _ => {}
                    }
                },
        }

        return Res::None;
    }
}


struct Tree<'a, K, P>
    where K: PartialOrd + Debug
{
    diam: u32,
    root: Node<'a, K, P>,
}

impl<'a, K, P> Tree<'a, K, P>
    where K: PartialOrd + Debug
{
    pub fn search(&self, key: &K) -> Option<&P> {
        let mut node = &self.root;
        loop {
            match node.search(key) {
                Res::None => return None,
                Res::Found(i) => return node.get_pointer(i),
                Res::Down(i) => match node.get_node(i) {
                    Some(nd) => node = nd,
                    None => return None,
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::store::trees::b_tree::Node::{Node, Leaf};
    use crate::store::trees::b_tree::Tree;

    #[test]
    fn simple_test() {
        let node_8 = Leaf {
            keys: vec![16, 19],
            pts: vec![8, 8, 8],
            link: None,
        };
        let node_7 = Leaf {
            keys: vec![14, 15],
            pts: vec![7, 7, 7],
            link: None,
        };
        let node_6 = Leaf {
            keys: vec![12],
            pts: vec![6, 6, 6],
            link: None,
        };
        let node_5 = Leaf {
            keys: vec![9, 10, 11],
            pts: vec![5, 5, 5],
            link: None,
        };
        let node_4 = Leaf {
            keys: vec![8],
            pts: vec![4, 4, 4],
            link: None,
        };
        let node_3 = Leaf {
            keys: vec![7],
            pts: vec![3, 3, 3],
            link: None,
        };
        let node_2 = Leaf {
            keys: vec![6],
            pts: vec![2, 2, 2],
            link: None,
        };
        let node_1 = Leaf {
            keys: vec![2, 3, 4],
            pts: vec![1, 1, 1],
            link: None,
        };

        let node_i_1 = Node { keys: vec![4, 6], links: vec![node_1, node_2, node_3] };
        let node_i_2 = Node { keys: vec![8, 11], links: vec![node_4, node_5, node_6] };
        let node_i_3 = Node { keys: vec![15, 19], links: vec![node_7, node_8] };
        let root = Node { keys: vec![7, 12], links: vec![node_i_1, node_i_2, node_i_3] };

        let tree = Tree { diam: 3, root };

        if let Some(p) = (&tree).search(&10) {
            assert_eq!(p, &5)
        } else {
            panic!("")
        }
    }
}