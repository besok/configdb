use std::rc::Rc;
use rand::distributions::{Uniform, Distribution};
use rand::prelude::ThreadRng;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::cmp::Ordering::Greater;
use std::cmp::Ordering::Less;
use crate::store::memory::skip_list::SearchResult::NotFound;
use crate::store::memory::skip_list::SearchResult::Down;
use crate::store::memory::skip_list::SearchResult::Forward;
use crate::store::memory::skip_list::SearchResult::Found;
use std::cell::RefCell;
use std::fmt::Debug;

type SkipNode<K: Ord + Clone + Debug, V: Clone + Debug> = Rc<RefCell<Node<K, V>>>;

struct LevelGenerator {
    total: usize,
    p: f64,
    sampler: Uniform<f64>,
    rand: ThreadRng,
}

impl LevelGenerator {
    fn new(total: usize) -> Self {
        let sampler = rand::distributions::uniform::Uniform::new(0.0f64, 1.0);
        let rand = rand::thread_rng();
        LevelGenerator { total, sampler, rand, p: 0.5 }
    }
    fn random(&mut self) -> usize {
        let mut height = 0;
        let mut temp = self.p;
        let level = 1.0 - self.sampler.sample(&mut self.rand);

        while temp > level && height + 1 < self.total {
            height += 1;
            temp *= self.p
        }
        height
    }
}

#[derive(Debug)]
struct Head<K: Ord + Clone + Debug, V: Clone + Debug> {
    next: Option<Rc<RefCell<Node<K, V>>>>
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> Head<K, V> {
    pub fn new(next: Option<Rc<RefCell<Node<K, V>>>>) -> Self {
        Head { next }
    }
    pub fn empty() -> Self {
        Head { next: None }
    }


    fn try_upd_head(&mut self, node: Rc<RefCell<Node<K, V>>>) {
        match &self.next {
            None => self.next = Some(node),
            Some(n) => {
                if let Some(Greater) = Node::compare_by_key(n.clone(), node.clone()) {
                    match Node::compare_by_level(n.clone(), node.clone()) {
                        Some(Less) | Some(Equal) => self.next = Some(node),
                        _ => ()
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct Node<K: Ord + Clone + Debug, V: Clone + Debug> {
    key: K,
    val: V,
    level: usize,
    next: Option<SkipNode<K, V>>,
    prev: Option<SkipNode<K, V>>,
    under: Option<SkipNode<K, V>>,
}

enum SearchResult<K: Ord + Clone + Debug, V: Clone + Debug> {
    Forward(SkipNode<K, V>),
    Down((SkipNode<K, V>)),
    NotFound,
    Found(V),
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> Node<K, V> {
    fn new(key: K, val: V, level: usize) -> Self {
        Node { key, val, level, under: None, next: None, prev: None }
    }
    fn new_with(key: K, val: V, level: usize) -> SkipNode<K, V> {
        Rc::new(RefCell::new(Node::new(key, val, level)))
    }
    fn set_under(&mut self, under: Option<SkipNode<K, V>>) {
        self.under = under;
    }

    fn set_under_to_node(node: SkipNode<K, V>, under: SkipNode<K, V>) {
        RefCell::borrow_mut(&node).set_under(Some(under))
    }

    fn value(&self) -> V {
        self.val.clone()
    }

    fn set_value(&mut self, val: V) {
        self.val = val.clone();
        if let Some(under) = &self.under {
            RefCell::borrow_mut(under).set_value(val.clone());
        }
    }

    fn next(node: SkipNode<K, V>) -> Option<SkipNode<K, V>> {
        node.borrow().next.as_ref().map(|n| n.clone())
    }
    fn prev(node: SkipNode<K, V>) -> Option<SkipNode<K, V>> {
        node.borrow().prev.as_ref().map(|n| n.clone())
    }


    fn set_next(node: SkipNode<K, V>, next_node: SkipNode<K, V>) {
        match Node::next(node.clone()) {
            None => {
                node.borrow_mut().next = Some(next_node.clone());
                next_node.borrow_mut().prev = Some(node.clone());
            },
            Some(old_next) => {
                node.borrow_mut().next = Some(next_node.clone());
                next_node.borrow_mut().prev = Some(node.clone());
                next_node.borrow_mut().next = Some(old_next.clone());
                old_next.borrow_mut().prev = Some(next_node.clone());
            }
        }
    }

    fn set_prev(node: SkipNode<K, V>, prev_node: SkipNode<K, V>) {
        match Node::prev(node.clone()) {
            None => {
                node.borrow_mut().prev = Some(prev_node.clone());
                prev_node.borrow_mut().next = Some(node.clone());
            }
            Some(old_prev) => {
                node.borrow_mut().prev = Some(prev_node.clone());
                prev_node.borrow_mut().next = Some(node.clone());
                prev_node.borrow_mut().prev = Some(old_prev.clone());
                old_prev.borrow_mut().next = Some(prev_node.clone());
            }
        }
    }


    fn compare(&self, key: &K) -> SearchResult<K, V> {
        match (self.key.partial_cmp(key), self.level) {
            (Some(Equal), _) => SearchResult::Found(self.val.clone()),
            (Some(Less), e) if e > 1 =>
                match (&self.next, &self.under) {
                    (Some(n), _) => Forward(n.clone()),
                    (None, Some(under)) => Down(under.clone()),
                    (None, None) => NotFound,
                },
            (Some(Greater), e) if e > 1 => match &self.prev {
                Some(prev) => match &RefCell::borrow(prev).under {
                    Some(under) => Down(under.clone()),
                    None => NotFound
                },
                None => match &self.under {
                    Some(under) => SearchResult::Down(under.clone()),
                    None => NotFound,
                },
            },
            (_, _) => NotFound
        }
    }
    fn set_val_to_node(node: SkipNode<K, V>, val: V) {
        node.borrow_mut().set_value(val);
    }
    fn compare_by_key(left: SkipNode<K, V>, right: SkipNode<K, V>) -> Option<Ordering> {
        let right_key = &RefCell::borrow(&right).key;
        let left_key = &RefCell::borrow(&left).key;
        left_key.partial_cmp(right_key)
    }
    fn compare_by_level(left: SkipNode<K, V>, right: SkipNode<K, V>) -> Option<Ordering> {
        let right_key = &RefCell::borrow(&right).level;
        let left_key = &RefCell::borrow(&left).level;
        left_key.partial_cmp(right_key)
    }
    fn connect(left: SkipNode<K, V>, right: SkipNode<K, V>) {
        match Node::compare_by_key(left.clone(), right.clone()) {
            Some(Ordering::Less) => Node::set_next(left.clone(), right.clone()),
            Some(Ordering::Greater) => Node::set_prev(left.clone(), right.clone()),
            _ => (),
        }
    }
    fn is_under(node: Rc<RefCell<Node<K, V>>>) -> bool {
        RefCell::borrow(&node.clone()).under.is_some()
    }
    fn get_under(node: Rc<RefCell<Node<K, V>>>) -> Rc<RefCell<Node<K, V>>> {
        RefCell::borrow(&node).under.as_ref().unwrap().clone()
    }
}

struct SkipList<K: Ord + Clone + Debug, V: Clone + Debug> {
    head: RefCell<Head<K, V>>,
    levels: usize,
    generator: LevelGenerator,
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> SkipList<K, V> {
    pub fn new() -> Self {
        SkipList::with_capacity(66_000)
    }
    pub fn with_capacity(exp_cap: usize) -> Self {
        let levels = (exp_cap as f64).log2().floor() as usize;
        let head = RefCell::new(Head::new(None));
        let generator = LevelGenerator::new(levels);

        SkipList { head, levels, generator }
    }
    pub fn search(&self, key: &K) -> Option<V> {
        match &self.first() {
            Some(n) => self.search_in(n.clone(), key),
            _ => None
        }
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        if self.head.borrow().next.is_none() {
            let mut curr_node = Node::new_with(key.clone(), val.clone(), self.levels);
            &self.head.borrow_mut().try_upd_head(curr_node.clone());
            let mut cur_lvl = self.levels - 1;

            while cur_lvl > 0 {
                let under_node = Node::new_with(key.clone(), val.clone(), cur_lvl);
                Node::set_under_to_node(curr_node, under_node.clone());
                curr_node = under_node.clone();
                cur_lvl = cur_lvl - 1
            }
            None
        } else {
            let first_node = self.first();
            let mut curr_node = first_node.as_ref().unwrap().clone();
            let mut path_stack: Vec<Rc<RefCell<Node<K, V>>>> = vec![];
            loop {
                let cmp = RefCell::borrow(&curr_node).compare(&key);
                match cmp {
                    Forward(next) => curr_node = next.clone(),
                    NotFound => {
                        let mut new_low_node = Node::new_with(key.clone(), val.clone(), 1);

                        Node::connect(new_low_node.clone(), curr_node.clone());

                        let mut curr_lvl: usize = 2;
                        let total_lvl = self.generator.random() + 1;
                        while curr_lvl <= total_lvl {
                            let new_node = Node::new_with(key.clone(), val.clone(), curr_lvl);

                            new_node.borrow_mut().set_under(Some(new_low_node.clone()));
                            let neigh_node = path_stack.pop().unwrap();

                            Node::connect(new_node.clone(), neigh_node.clone());

                            new_low_node = new_node.clone();
                            curr_lvl = curr_lvl + 1;
                        }

                        self.head.borrow_mut().try_upd_head(new_low_node);
                        return None;
                    }
                    Down(under) => {
                        path_stack.push(under.clone());
                        curr_node = under.clone();
                    }
                    Found(old_v) => {
                        curr_node.borrow_mut().set_value(val);
                        return Some(old_v);
                    }
                }
            }
        }
    }


    fn search_in(&self, node: Rc<RefCell<Node<K, V>>>, key: &K) -> Option<V> {
        let mut curr_node = node.clone();
        loop {
            match RefCell::borrow(&curr_node.clone()).compare(key) {
                NotFound => return None,
                Forward(n) | Down(n) => curr_node = n.clone(),
                Found(v) => return Some(v),
            }
        }
    }

    fn first(&self) -> Option<Rc<RefCell<Node<K, V>>>> {
        RefCell::borrow(&self.head).next.as_ref().map(|v| v.clone())
    }
}


#[cfg(test)]
mod tests {
    use crate::store::memory::skip_list::{Node, LevelGenerator, SkipList};

    #[test]
    fn connect_node_test() {
        let left = Node::new_with(10, 10, 1);
        let mid = Node::new_with(20, 20, 1);
        let right = Node::new_with(30, 30, 1);

        Node::connect(left.clone(), right.clone());

        let nl_k = left.borrow().next.as_ref().unwrap().clone().borrow().key;
        let pr_k = right.borrow().prev.as_ref().unwrap().clone().borrow().key;
        assert_eq!(nl_k, 30);
        assert_eq!(pr_k, 10);

        Node::connect(mid.clone(), left.clone());
        Node::connect(mid.clone(), right.clone());

        let l_n_k = left.borrow().next.as_ref().unwrap().clone().borrow().key;
        let m_p_k = mid.borrow().prev.as_ref().unwrap().clone().borrow().key;
        let m_n_k = mid.borrow().next.as_ref().unwrap().clone().borrow().key;
        let r_p_k = right.borrow().prev.as_ref().unwrap().clone().borrow().key;
        assert_eq!(l_n_k, 20);
        assert_eq!(m_p_k, 10);
        assert_eq!(m_n_k, 30);
        assert_eq!(r_p_k, 20);


    }


    #[test]
    fn simple_test() {
        let node = Node::new(10, 20, 3);
        assert_eq!(node.val, 20)
    }

    #[test]
    fn skip_list_test() {
        let mut list: SkipList<u64, u64> = SkipList::with_capacity(4000_000_000);
        let _ = list.insert(20, 20);
        let _ = list.insert(20, 20);
        let _ = list.insert(10, 10);
        let _ = list.insert(10, 10);
        let _ = list.insert(30, 30);
        let _ = list.insert(40, 40);
        let _ = list.insert(50, 50);
        let _ = list.insert(1, 1);
        let _ = list.insert(15, 15);
        let _ = list.insert(200, 200);
        let _ = list.insert(60, 60);
        let _ = list.insert(70, 70);
        let _ = list.insert(80, 80);
        let _ = list.insert(80, 800);
        let _ = list.insert(2, 2);


        let res = list.search(&10);
        assert_eq!(res.is_some(), true);
        assert_eq!(res, Some(10));
        let res = list.search(&70);
        assert_eq!(res.is_some(), true);
        assert_eq!(res, Some(70));
    }


    #[test]
    fn double_insert_test() {
        let mut list: SkipList<u64, u64> = SkipList::new();
        let opt1 = list.insert(10, 10);
        let opt2 = list.insert(10, 11);
        assert_eq!(opt1, None);
        assert_eq!(opt2, Some(10));
    }

    #[test]
    fn simple_skip_list_test() {
        let mut list: SkipList<u64, u64> = SkipList::with_capacity(4000_000_000);
        let opt = list.insert(10, 10);
        assert_eq!(opt.is_none(), true);
        assert_eq!(list.levels, 31);
        println!(" {:?} ", list.head);

        let opt = list.insert(10, 100);
        assert_eq!(opt.is_none(), false);
        assert_eq!(opt.unwrap(), 10);
    }

    #[test]
    fn rand_test() {
        let mut gen = LevelGenerator::new(16);
        for _ in 0..100 {
            assert_eq!(true, gen.random() >= 0)
        }
    }
}

