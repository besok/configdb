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
                for (i, k) in keys.iter().enumerate() {
                    match key.partial_cmp(k) {
                        Some(Ordering::Equal) |
                        Some(Ordering::Less) => return SearchRes::Down(i),
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


struct Tree<K, P>
    where K: PartialOrd + Debug
{
    diam: usize,
    root: Rc<Node<K, P>>,
}

impl<K, P> Tree<K, P>
    where K: PartialOrd + Debug
{
    pub fn new(diam: usize, root: Node<K, P>) -> Self {
        Tree { diam, root: Rc::new(root) }
    }
    fn search(&self, key: &K) -> Option<Rc<P>> {
        let mut node = self.root.clone();
        loop {
            match node.search(key) {
                SearchRes::Down(i) =>
                    match node.get_node(i) {
                        Some(nd) => node = nd,
                        None => return None,
                    },
                SearchRes::Found(i) => return node.get_ptr(i),
                SearchRes::None => return None,
            }
        }
    }

    fn build_path(&self, key: K) -> InsertStack<K,P> {
        let mut node = self.root.clone();
        let mut stack = InsertStack::new();
        loop {
            stack.push(node.clone());
            match node.search(&key) {
                SearchRes::Down(i) =>
                    match node.get_node(i) {
                        Some(nd) => node = nd,
                        None => return stack
                    },
                SearchRes::Found(i) => return stack,
                SearchRes::None => return stack
            }
        }
    }
}

struct InsertStack<K, P>
    where K: PartialOrd + Debug
{
    values: Vec<Rc<Node<K, P>>>
}

impl<K, P> InsertStack<K, P> where K: PartialOrd + Debug {
    pub fn new() -> Self {
        InsertStack { values: vec![] }
    }
    pub fn push(&mut self, v: Rc<Node<K, P>>) {
        self.values.push(v)
    }
    pub fn pop(&mut self) -> Option<Rc<Node<K, P>>> {
        self.values.pop()
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
        let tree = tree();

        if let Some(_) = (&tree).search(&43){
            panic!("")
        }
        if let Some(e) = (&tree).search(&4){
            assert_eq!(e,Rc::new(4))
        }else{
            panic!("")
        }
        if let Some(e) = (&tree).search(&49){
            assert_eq!(e,Rc::new(49))
        }else{
            panic!("")
        }
    }

    fn tree() -> Tree<i32, i32> {
        let leaf_1 = Node::new_leaf(vec![1, 2, 4], vec![1, 2, 4]);
        let leaf_2 = Node::new_leaf(vec![6, 8, 9, 10], vec![6, 8, 9, 10]);
        let leaf_3 = Node::new_leaf(vec![12, 14, 16, 17], vec![12, 14, 16, 17]);
        let leaf_4 = Node::new_leaf(vec![20, 22, 24], vec![20, 22, 24]);
        let leaf_5 = Node::new_leaf(vec![27, 28, 32], vec![27, 28, 32]);
        let leaf_6 = Node::new_leaf(vec![34, 38, 39, 41], vec![34, 38, 39, 41]);
        let leaf_7 = Node::new_leaf(vec![44, 47, 49], vec![44, 47, 49]);
        let leaf_8 = Node::new_leaf(vec![50, 60, 70], vec![50, 60, 70]);

        let node_1 = Node::new_node(vec![6], vec![leaf_1, leaf_2]);
        let node_2 = Node::new_node(vec![20, 27, 34], vec![leaf_3, leaf_4, leaf_5, leaf_6]);
        let node_3 = Node::new_node(vec![50], vec![leaf_7, leaf_8]);

        let root = Node::new_node(vec![12, 44], vec![node_1, node_2, node_3]);
        Tree::new(4, root)
    }

    #[test]
    fn simple_stack_test(){
        let tr = tree();
        let stack = (&tr).build_path(80);
        for el in stack.values.iter(){
            println!("{:?}",el)
        }
    }
}