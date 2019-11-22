use std::cmp::Ordering;
use std::fmt::Debug;

enum SearchRes {
    Down(usize),
    Found(usize),
    None,
}

enum InsertRes<'a, K, P>
    where K: PartialOrd + Debug
{
    Ready(&'a mut Node<'a, K, P>),
    Full(&'a mut Node<'a, K, P>),
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
    fn keys(&'a self) -> &'a Vec<K> {
        match self {
            Node::Node { keys, .. } |
            Node::Leaf { keys, .. } => keys,
        }
    }
    fn is_full(&self, br_f: usize) -> bool {
        match self {
            Node::Node { keys, .. } => keys.len() == br_f - 1,
            Node::Leaf { keys, .. } => keys.len() == br_f
        }
    }


    fn get_node_mut(&'a mut self, i: usize) -> Option<&'a mut Node<'a, K, P>> {
        match self {
            Node::Node { links, .. } => links.get_mut(i),
            _ => None
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
    fn insert(&mut self, key: K, p: P) -> Result<(), String> {
        match self {
            Node::Node { .. } => Err(String::from("only for leafs")),
            Node::Leaf { keys, pts, .. } => {
                for (i, el) in keys.iter().enumerate() {
                    match key.partial_cmp(el) {
                        Some(Ordering::Less) => {
                            keys.insert(i, key);
                            pts.insert(i, p);
                            return Ok(());
                        }
                        Some(Ordering::Greater) => {}
                        Some(Ordering::Equal) => {
                            keys[i] = key;
                            pts[i] = p;
                            return Ok(());
                        }
                        None => {}
                    }
                }
                keys.push(key);
                pts.push(p);
                Ok(())
            }
        }
    }
    fn search_leaf(&'a mut self, key: &K) -> Option<&'a mut Node<'a, K, P>> {
        let mut node = self;
        loop {
            match node {
                Node::Leaf { .. } => return Some(node),
                Node::Node { .. } =>
                    match node.search(key) {
                        SearchRes::Down(i) => match node.get_node_mut(i) {
                            Some(nd) => node = nd,
                            None => return None,
                        },
                        _ => return None
                    }
            }
        }
    }

    fn search(&self, key: &K) -> SearchRes {
        match self {
            Node::Node { keys, .. } => {
                for (i, k) in keys.iter().enumerate() {
                    match key.partial_cmp(k) {
                        Some(Ordering::Equal) |
                        Some(Ordering::Less) => {
                            return SearchRes::Down(i);
                        }
                        _ => {}
                    }
                }
                return SearchRes::Down(keys.len());
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


struct Tree<'a, K, P>
    where K: PartialOrd + Debug
{
    diam: usize,
    root: Node<'a, K, P>,
}

impl<'a, K, P> Tree<'a, K, P>
    where K: PartialOrd + Debug
{
    pub fn search(&self, key: &K) -> Option<&P> {
        let mut node = &self.root;
        loop {
            match node.search(key) {
                SearchRes::None => return None,
                SearchRes::Found(i) => return node.get_pointer(i),
                SearchRes::Down(i) =>
                    if let Some(nd) = node.get_node(i) {
                        node = nd
                    } else { return None; }
            }
        }
    }

    fn search_leaf(&'a mut self, key: &K) -> InsertRes<'a, K, P> {
        match self.root.search_leaf(key) {
            Some(e) => if e.keys().len() < self.diam - 1 {
                InsertRes::Ready(e)
            } else {
                InsertRes::Full(e)
            },
            None => InsertRes::None,
        }
    }

    fn insert(&'a mut self, key: K, ptr: P) -> Result<(), String> {
        match self.search_leaf(&key) {
            InsertRes::Ready(e) => return e.insert(key, ptr),
            InsertRes::Full(_e) => {}
            InsertRes::None => {}
        }

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use crate::store::trees::b_tree::Node::{Node, Leaf};
    use crate::store::trees::b_tree::Tree;
    use crate::store::trees::b_tree::InsertRes::Ready;

    #[test]
    fn simple_find_test() {
        let node_8 = Leaf {
            keys: vec![16, 19],
            pts: vec![8, 8, 8],
            link: None,
        };
        let node_7 = Leaf {
            keys: vec![14, 15],
            pts: vec![7, 7, ],
            link: None,
        };
        let node_6 = Leaf {
            keys: vec![12],
            pts: vec![6, ],
            link: None,
        };
        let node_5 = Leaf {
            keys: vec![9, 10, 11],
            pts: vec![5, 5, 5],
            link: None,
        };
        let node_4 = Leaf {
            keys: vec![8],
            pts: vec![4, ],
            link: None,
        };
        let node_3 = Leaf {
            keys: vec![7],
            pts: vec![3, ],
            link: None,
        };
        let node_2 = Leaf {
            keys: vec![6],
            pts: vec![2, ],
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

    #[test]
    fn insert_test() {
        let mut leaf = Leaf {
            keys: vec![16, 19],
            pts: vec![8, 8],
            link: None,
        };
        let mut node = Node { keys: vec![4, 6], links: vec![] };

        if let Ok(()) = node.insert(1, 1) {
            panic!("should be err")
        }

        if let (
            Ok(()),
            Some(p),
            vecs,
        ) = (
            leaf.insert(15, 9),
            leaf.get_pointer(2),
            leaf.keys(),
        ) {
            assert_eq!(p, &8);
            assert_eq!(vecs, &vec![15, 16, 19]);
        } else {
            panic!("")
        }

        let mut leaf = Leaf {
            keys: vec![16, 19],
            pts: vec![8, 8],
            link: None,
        };

        if let (Ok(()), Some(p), vecs, ) = (
            leaf.insert(16, 9),
            leaf.get_pointer(0),
            leaf.keys(),
        ) {
            assert_eq!(p, &9);
            assert_eq!(vecs, &vec![16, 19]);
        } else {
            panic!("")
        }

        let mut leaf = Leaf {
            keys: vec![16, 19],
            pts: vec![8, 8],
            link: None,
        };

        if let (Ok(()), Some(p), vecs, ) = (
            leaf.insert(17, 9),
            leaf.get_pointer(1),
            leaf.keys(),
        ) {
            assert_eq!(p, &9);
            assert_eq!(vecs, &vec![16, 17, 19]);
        } else {
            panic!("")
        }

        let mut leaf = Leaf {
            keys: vec![16, 19],
            pts: vec![8, 8],
            link: None,
        };

        if let (Ok(()), Some(p), vecs, ) = (
            leaf.insert(21, 9),
            leaf.get_pointer(2),
            leaf.keys(),
        ) {
            assert_eq!(p, &9);
            assert_eq!(vecs, &vec![16, 19, 21]);
        } else {
            panic!("")
        }
    }

    #[test]
    fn search_leaf() {
        let node_8 = Leaf {
            keys: vec![26, 27],
            pts: vec![8, 8],
            link: None,
        };
        let node_7 = Leaf {
            keys: vec![21, 23],
            pts: vec![7, 7, ],
            link: None,
        };
        let node_6 = Leaf {
            keys: vec![12],
            pts: vec![6, ],
            link: None,
        };
        let node_5 = Leaf {
            keys: vec![9, 10, 11],
            pts: vec![5, 5, 5],
            link: None,
        };
        let node_4 = Leaf {
            keys: vec![8],
            pts: vec![4, ],
            link: None,
        };
        let node_3 = Leaf {
            keys: vec![7],
            pts: vec![3, ],
            link: None,
        };
        let node_2 = Leaf {
            keys: vec![6],
            pts: vec![2, ],
            link: None,
        };
        let node_1 = Leaf {
            keys: vec![2, 3, 4],
            pts: vec![1, 1, 1],
            link: None,
        };

        let node_i_1 = Node { keys: vec![4, 6], links: vec![node_1, node_2, node_3] };
        let node_i_2 = Node { keys: vec![8, 11], links: vec![node_4, node_5, node_6] };
        let node_i_3 = Node { keys: vec![23, 27], links: vec![node_7, node_8] };
        let root = Node { keys: vec![7, 12], links: vec![node_i_1, node_i_2, node_i_3] };

        let mut tree = Tree { diam: 3, root };

//        println!(" -- ");
//        if let Some(_) = tree.search_leaf(&29) { panic!("") }
//        println!(" -- ");
//        if let Some(leaf_1) = tree.search_leaf(&9) {
//            assert_eq!(leaf_1.keys(), &vec![9, 10, 11]);
//        } else { panic!("") }
//        println!(" -- ");
        if let Ready(leaf_1) = tree.search_leaf(&14) {
            assert_eq!(leaf_1.keys(), &vec![21, 23]);
        } else { panic!("") }
    }

    #[test]
    fn insert_to_tree_test() {
        let node_8 = Leaf {
            keys: vec![26, 27],
            pts: vec![8, 8],
            link: None,
        };
        let node_7 = Leaf {
            keys: vec![21, 23],
            pts: vec![7, 7, ],
            link: None,
        };
        let node_6 = Leaf {
            keys: vec![12],
            pts: vec![6, ],
            link: None,
        };
        let node_5 = Leaf {
            keys: vec![9, 10, 11],
            pts: vec![5, 5, 5],
            link: None,
        };
        let node_4 = Leaf {
            keys: vec![8],
            pts: vec![4, ],
            link: None,
        };
        let node_3 = Leaf {
            keys: vec![7],
            pts: vec![3, ],
            link: None,
        };
        let node_2 = Leaf {
            keys: vec![6],
            pts: vec![2, ],
            link: None,
        };
        let node_1 = Leaf {
            keys: vec![2, 3, 4],
            pts: vec![1, 1, 1],
            link: None,
        };

        let node_i_1 = Node { keys: vec![4, 6], links: vec![node_1, node_2, node_3] };
        let node_i_2 = Node { keys: vec![8, 11], links: vec![node_4, node_5, node_6] };
        let node_i_3 = Node { keys: vec![23, 27], links: vec![node_7, node_8] };
        let root = Node { keys: vec![7, 12], links: vec![node_i_1, node_i_2, node_i_3] };

        let mut tree = Tree { diam: 3, root };

        if let Ok(()) = tree.insert(22, 1) {
            if let Some(p) = &tree.search(&22) {
                assert_eq!(p, &1)
            } else {
                panic!("")
            }
        } else {
            panic!("")
        }
    }
}