use std::rc::Rc;
use rand::distributions::{Uniform, Distribution};
use rand::prelude::ThreadRng;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::cmp::Ordering::Greater;
use std::cmp::Ordering::Less;
use crate::store::memory::skip_list::SearchResult::{NotFound, Backward};
use crate::store::memory::skip_list::SearchResult::Down;
use crate::store::memory::skip_list::SearchResult::Forward;
use crate::store::memory::skip_list::SearchResult::Found;
use crate::store::memory::skip_list::PrevSearchStep::FromAbove;
use crate::store::memory::skip_list::PrevSearchStep::FromLeft;
use crate::store::memory::skip_list::PrevSearchStep::FromHead;
use std::cell::RefCell;


type SkipNode<K: Ord + Clone, V: Clone> = Rc<RefCell<Node<K, V>>>;

struct LevelGenerator {
    p: f64,
    sampler: Uniform<f64>,
    rand: ThreadRng,
}

impl LevelGenerator {
    fn new() -> Self {
        LevelGenerator {
            sampler: Uniform::new(0.0f64, 1.0),
            rand: rand::thread_rng(),
            p: 0.5,
        }
    }
    fn random(&mut self, total: usize) -> usize {
        let mut height = 0;
        let mut temp = self.p;
        let level =
            1.0 - self.sampler.sample(&mut self.rand);

        while temp > level && height + 1 < total {
            height += 1;
            temp *= self.p
        }
        height
    }
}

struct Head<K: Ord + Clone, V: Clone> {
    next: Option<SkipNode<K, V>>
}

impl<K: Ord + Clone, V: Clone> Head<K, V> {
    pub fn new(next: Option<SkipNode<K, V>>) -> Self {
        Head { next }
    }
    pub fn empty() -> Self {
        Head { next: None }
    }
    fn try_upd_head(&mut self, node: SkipNode<K, V>) {
        match &self.next {
            None => self.next = Some(node),
            Some(n) =>
                if let Some(Greater) = Node::cmp_by_key(n.clone(), node.clone()) {
                    match Node::cmp_by_lvl(n.clone(), node.clone()) {
                        Some(Less) | Some(Equal) => self.next = Some(node),
                        _ => ()
                    }
                },
        }
    }
}


struct Node<K: Ord + Clone, V: Clone> {
    key: K,
    val: V,
    level: usize,
    next: Option<SkipNode<K, V>>,
    prev: Option<SkipNode<K, V>>,
    under: Option<SkipNode<K, V>>,
}

enum PrevSearchStep {
    FromAbove,
    FromLeft,
    FromHead,
}


enum SearchResult<K: Ord + Clone, V: Clone> {
    Forward(SkipNode<K, V>),
    Backward(SkipNode<K, V>),
    Down(SkipNode<K, V>),
    Found(V),
    NotFound,
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    fn new(key: K, val: V, level: usize) -> Self {
        Node { key, val, level, under: None, next: None, prev: None }
    }
    fn new_with(key: K, val: V, level: usize) -> SkipNode<K, V> {
        Rc::new(RefCell::new(Node::new(key, val, level)))
    }
    fn new_in_list(key: K,
                   val: V,
                   total_lvl: usize,
                   curr_node: Option<SkipNode<K, V>>,
                   path: &mut Vec<SkipNode<K, V>>) -> SkipNode<K, V> {
        let mut new_low_node = Node::new_with(key.clone(), val.clone(), 1);
        if curr_node.is_some() {
            Node::connect_new(curr_node.unwrap().clone(), new_low_node.clone());
        }

        let mut curr_lvl: usize = 2;
        while curr_lvl <= total_lvl {
            let new_node = Node::new_with(key.clone(), val.clone(), curr_lvl);
            RefCell::borrow_mut(&new_node).under = Some(new_low_node);
            if let Some(neigh_node) = path.pop() {
                Node::connect_new(neigh_node.clone(), new_node.clone());
            }

            new_low_node = new_node.clone();
            curr_lvl = curr_lvl + 1;
        }

        new_low_node.clone()
    }
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    fn cmp_by_key(left: SkipNode<K, V>, right: SkipNode<K, V>) -> Option<Ordering> {
        let right_key = &RefCell::borrow(&right).key;
        let left_key = &RefCell::borrow(&left).key;
        left_key.partial_cmp(right_key)
    }
    fn cmp_by_lvl(left: SkipNode<K, V>, right: SkipNode<K, V>) -> Option<Ordering> {
        let right_key = &RefCell::borrow(&right).level;
        let left_key = &RefCell::borrow(&left).level;
        left_key.partial_cmp(right_key)
    }
    fn compare(&self, key: &K, prev_step: &PrevSearchStep) -> SearchResult<K, V> {
        match self.key.partial_cmp(key) {
            Some(Equal) => SearchResult::Found(self.val.clone()),
            Some(Less) =>
                match (&self.next, &self.under) {
                    (Some(n), _) => Forward(n.clone()),
                    (None, Some(under)) => Down(under.clone()),
                    (None, None) => NotFound,
                },
            Some(Greater) =>
                match (&self.prev, &self.under) {
                    (Some(prev), _) =>
                        match (RefCell::borrow(prev).under.as_ref(), prev_step) {
                            (Some(prev_under), FromLeft) => Down(prev_under.clone()),
                            (_, FromAbove) => Backward(prev.clone()),
                            (_, _) => NotFound
                        },
                    (None, Some(under)) => Down(under.clone()),
                    (None, None) => NotFound
                },
            None => NotFound
        }
    }
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    fn get_next(node: SkipNode<K, V>) -> Option<SkipNode<K, V>> {
        node.borrow().next.as_ref().map(|n| n.clone())
    }
    fn get_prev(node: SkipNode<K, V>) -> Option<SkipNode<K, V>> {
        node.borrow().prev.as_ref().map(|n| n.clone())
    }
    fn set_next(node: SkipNode<K, V>, next_node: SkipNode<K, V>) {
        match Node::get_next(node.clone()) {
            None => {
                node.borrow_mut().next = Some(next_node.clone());
                next_node.borrow_mut().prev = Some(node.clone());
            }
            Some(old_next) => {
                node.borrow_mut().next = Some(next_node.clone());
                next_node.borrow_mut().prev = Some(node.clone());
                next_node.borrow_mut().next = Some(old_next.clone());
                old_next.borrow_mut().prev = Some(next_node.clone());
            }
        }
    }
    fn set_prev(node: SkipNode<K, V>, prev_node: SkipNode<K, V>) {
        match Node::get_prev(node.clone()) {
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
    fn connect_new(node: SkipNode<K, V>, new_node: SkipNode<K, V>) {
        match Node::cmp_by_key(node.clone(), new_node.clone()) {
            Some(Ordering::Less) => Node::set_next(node.clone(), new_node.clone()),
            Some(Ordering::Greater) => Node::set_prev(node.clone(), new_node.clone()),
            _ => (),
        }
    }
    fn set_value(&mut self, val: V) {
        self.val = val.clone();
        if let Some(under) = &self.under {
            RefCell::borrow_mut(under).set_value(val.clone());
        }
    }

    fn find_first(node: SkipNode<K, V>) -> SkipNode<K, V> {
        let mut first_node = node.clone();
        if RefCell::borrow(&node.clone()).prev.is_some() {
            let mut prev_node = RefCell::borrow(&node).prev.clone();
            while prev_node.is_some() {
                first_node = prev_node.clone().unwrap();
                prev_node = RefCell::borrow(&prev_node.unwrap()).prev.clone();
            }
        }
        first_node.clone()
    }
}

struct SkipList<K: Ord + Clone, V: Clone> {
    head: RefCell<Head<K, V>>,
    levels: usize,
    size: usize,
    generator: LevelGenerator,
}

impl<K: Ord + Clone, V: Clone> SkipList<K, V> {
    pub fn new() -> Self {
        SkipList::with_capacity(66_000)
    }
    pub fn with_capacity(exp_cap: usize) -> Self {
        let levels = (exp_cap as f64).log2().floor() as usize;
        let head = RefCell::new(Head::new(None));
        let generator = LevelGenerator::new();
        let size = 0;
        SkipList { head, levels, generator, size }
    }
    pub fn search(&self, key: &K) -> Option<V> {
        match &self.first() {
            Some(n) => self.search_in(n.clone(), key),
            _ => None
        }
    }

    pub fn iter(&self) -> SkipListIterator<K, V> {
        SkipListIterator::new(self)
    }
    pub fn iter_low_level(&self) -> SkipListDistinctIterator<K, V> {
        SkipListDistinctIterator::new(self)
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        if self.head.borrow().next.is_none() {
            let new_node = Node::new_in_list(
                key, val, self.levels, None, &mut vec![]);
            self.head.borrow_mut().try_upd_head(new_node);
            self.inc_size();
            None
        } else {
            let first_node = self.first();
            let mut curr = first_node.as_ref().unwrap().clone();
            let mut prev_step = FromHead;
            let mut path: Vec<Rc<RefCell<Node<K, V>>>> = vec![];
            loop {
                let cmp_with_curr_node = RefCell::borrow(&curr).compare(&key, &prev_step);
                match cmp_with_curr_node {
                    Backward(prev) => {
                        curr = prev.clone();
                        prev_step = FromLeft;
                    }
                    Forward(next) => {
                        curr = next.clone();
                        prev_step = FromLeft;
                    }
                    NotFound => {
                        let lev = self.generator.random(self.levels) + 1;
                        let new_node =
                            Node::new_in_list(key, val, lev, Some(curr.clone()), &mut path);
                        self.head.borrow_mut().try_upd_head(new_node);
                        self.inc_size();
                        return None;
                    }
                    Down(under) => {
                        path.push(curr.clone());
                        curr = under.clone();
                        prev_step = FromAbove;
                    }
                    Found(old_v) => {
                        curr.borrow_mut().set_value(val);
                        return Some(old_v);
                    }
                }
            }
        }
    }

    fn inc_size(&mut self) {
        self.size = self.size + 1
    }
    fn dec_size(&mut self) {
        self.size = self.size - 1
    }
    fn search_in(&self, node: Rc<RefCell<Node<K, V>>>, key: &K) -> Option<V> {
        let mut curr_node = node.clone();
        let mut prev_step = FromHead;
        loop {
            match RefCell::borrow(&curr_node.clone()).compare(key, &prev_step) {
                NotFound => return None,
                Backward(p) => curr_node = p.clone(),
                Found(v) => return Some(v),
                Forward(n) => {
                    curr_node = n.clone();
                    prev_step = FromLeft;
                }
                Down(n) => {
                    curr_node = n.clone();
                    prev_step = FromAbove;
                }
            }
        }
    }

    fn first(&self) -> Option<SkipNode<K, V>> {
        RefCell::borrow(&self.head).next.as_ref().map(|v| v.clone())
    }
}

struct SkipListIterator<K: Ord + Clone, V: Clone> {
    size: usize,
    curr: Option<SkipNode<K, V>>,
}

struct SkipListDistinctIterator<K: Ord + Clone, V: Clone> {
    size: usize,
    curr: Option<SkipNode<K, V>>,
}

impl<K: Ord + Clone, V: Clone> SkipListDistinctIterator<K, V> {
    fn new(list: &SkipList<K, V>) -> Self {
        let size = list.size;
        let curr = match &list.first() {
            None => None,
            Some(n) => {
                let mut lower_node = n.clone();
                while RefCell::borrow(&lower_node).under.is_some() {
                    lower_node = RefCell::borrow(&lower_node.clone()).under.as_ref().unwrap().clone();
                }
                Some(Node::find_first(lower_node.clone()))
            }
        };

        SkipListDistinctIterator { size, curr }
    }

    fn next_opt(&self) -> Option<SkipNode<K, V>> {
        if self.curr.is_none() {
            None
        } else {
            RefCell::borrow(self.curr.as_ref().unwrap())
                .next.as_ref().map(|v| v.clone())
        }
    }
}

impl<K: Ord + Clone, V: Clone> Iterator for SkipListDistinctIterator<K, V> {
    type Item = SkipNode<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        match &self.next_opt() {
            None => {
                let old_curr = self.curr.clone();
                self.curr = None;
                old_curr
            }
            Some(n) => {
                let old_curr = self.curr.clone();
                self.curr = Some(n.clone());
                old_curr
            }
        }
    }
}

impl<K: Ord + Clone, V: Clone> SkipListIterator<K, V> {
    fn get_under(node: SkipNode<K, V>) -> Option<SkipNode<K, V>> {
        RefCell::borrow(&node).under.clone()
    }

    fn new(list: &SkipList<K, V>) -> Self {
        let size = list.size;
        let curr = match &list.first() {
            None => None,
            Some(n) => {
                let first_node = Node::find_first(n.clone());
                Some(first_node.clone())
            }
        };

        SkipListIterator { size, curr }
    }

    fn find_next(&self) -> Option<SkipNode<K, V>> {
        RefCell::borrow(self.curr.as_ref().unwrap()).next.as_ref().map(|v| v.clone())
    }

    fn find_under(&self) -> Option<SkipNode<K, V>> {
        RefCell::borrow(self.curr.as_ref().unwrap()).under.as_ref().map(|v| v.clone())
    }

    fn next_opt(&mut self) -> Option<SkipNode<K, V>> {
        match &self.find_next() {
            None => {
                match &self.find_under() {
                    None => None,
                    Some(under) => {
                        let first = Node::find_first(under.clone());
                        self.curr = Some(first.clone());
                        Some(first.clone())
                    }
                }
            }
            Some(next) => {
                self.curr = Some(next.clone());
                Some(next.clone())
            }
        }
    }
}

impl<K: Ord + Clone, V: Clone> Iterator for SkipListIterator<K, V> {
    type Item = SkipNode<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_opt()
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

        Node::connect_new(left.clone(), right.clone());

        let nl_k = left.borrow().next.as_ref().unwrap().clone().borrow().key;
        let pr_k = right.borrow().prev.as_ref().unwrap().clone().borrow().key;
        assert_eq!(nl_k, 30);
        assert_eq!(pr_k, 10);

        Node::connect_new(right.clone(), mid.clone());

        let l_n_k = left.borrow().next.as_ref().unwrap().clone().borrow().key;
        assert_eq!(l_n_k, 20);

        let m_p_k = mid.borrow().prev.as_ref().unwrap().clone().borrow().key;
        assert_eq!(m_p_k, 10);

        let m_n_k = mid.borrow().next.as_ref().unwrap().clone().borrow().key;
        assert_eq!(m_n_k, 30);

        let r_p_k = right.borrow().prev.as_ref().unwrap().clone().borrow().key;
        assert_eq!(r_p_k, 20);
    }


    #[test]
    fn simple_test() {
        let node = Node::new(10, 20, 3);
        assert_eq!(node.val, 20)
    }

    #[test]
    fn print_test() {
        let mut list: SkipList<u64, u64> = SkipList::with_capacity(16);
        let _ = list.insert(200, 200);
        let _ = list.insert(1, 1);
        let _ = list.insert(80, 800);
        let _ = list.insert(800, 800);
        let _ = list.insert(8, 800);


        for el in list.iter_low_level() {
            println!("{} - {}", el.borrow().key, el.borrow().level)
        }
    }


    #[test]
    fn skip_list_test() {
        let mut list: SkipList<u64, u64> = SkipList::with_capacity(16);
        let _ = list.insert(200, 200);
        let _ = list.insert(1, 1);
        let _ = list.insert(80, 800);
        let _ = list.insert(10, 10);
        let _ = list.insert(70, 70);
        let _ = list.insert(20, 2);
        let _ = list.insert(800, 800);


        test_search(list.search(&200), 200);
        test_search(list.search(&20), 2);
        test_search(list.search(&1), 1);
        test_search(list.search(&80), 800);
        test_search(list.search(&800), 800);
    }

    fn test_search(got_val: Option<u64>, exp_val: u64) {
        assert_eq!(got_val.is_some(), true);
        assert_eq!(got_val, Some(exp_val));
    }


    #[test]
    fn double_insert_test() {
        let mut list: SkipList<u64, u64> = SkipList::new();
        let opt1 = list.insert(10, 10);
        let opt2 = list.insert(10, 11);
        assert_eq!(opt1, None);
        assert_eq!(opt2, Some(10));

        let opt = list.search(&10);
        assert_eq!(opt.unwrap(), 11)
    }

    #[test]
    fn simple_skip_list_test() {
        let mut list: SkipList<u64, u64> = SkipList::with_capacity(4000_000_000);
        let opt = list.insert(10, 10);
        assert_eq!(opt.is_none(), true);
        assert_eq!(list.levels, 31);

        let opt = list.insert(10, 100);
        assert_eq!(opt.is_none(), false);
        assert_eq!(opt.unwrap(), 10);
    }

    #[test]
    fn rand_test() {
        let mut gen = LevelGenerator::new();
        for _ in 0..100000 {
            let i = gen.random(16);
            assert_eq!(true, i >= 0)
        }
    }
}

