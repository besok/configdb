use std::rc::Rc;
use rand::distributions::{Uniform, Distribution};
use rand::prelude::ThreadRng;
use std::cmp::Ordering;
use crate::store::memory::skip_list::SearchResult::NotFound;
use std::cell::RefCell;
use std::fmt::Debug;

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


    fn upd_head(&mut self, node: Rc<RefCell<Node<K, V>>>) {
        match &self.next {
            None => self.next = Some(node),
            Some(n) => {
                if let Some(Ordering::Greater) = Node::compare_nodes(n.clone(), node.clone()) {
                    self.next = Some(node)
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
    next: Option<Rc<RefCell<Node<K, V>>>>,
    under: Option<Rc<RefCell<Node<K, V>>>>,
}

enum SearchResult<V: Clone> {
    Forward,
    NotFound,
    Down,
    End,
    Found(V),
}

impl<K: Ord + Clone + Debug, V: Clone + Debug> Node<K, V> {
    fn new(key: K, val: V, level: usize) -> Self {
        Node { key, val, level, under: None, next: None }
    }
    fn new_with(key: K, val: V, level: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node::new(key, val, level)))
    }
    fn set_under(&mut self, under: Option<Rc<RefCell<Node<K, V>>>>) {
        self.under = under;
    }
    fn value(&self) -> V {
        self.val.clone()
    }

    fn insert(&mut self, val: V) {
        self.val = val.clone();
        if let Some(under) = &mut self.under {
            RefCell::borrow_mut(under).insert(val.clone());
        }
    }

    fn insert_next(&mut self, node: Rc<RefCell<Node<K, V>>>) {
        match &self.next {
            None => self.next = Some(node.clone()),
            Some(_) => {
                let old_next = self.next.as_ref().unwrap().clone();
                self.next = Some(node.clone());
                RefCell::borrow_mut(&node).next = Some(old_next)
            }
        }
    }

    fn compare(&self, key: &K) -> SearchResult<V> {
        match (self.key.partial_cmp(key), self.level) {
            (Some(Ordering::Equal), _) => SearchResult::Found(self.val.clone()),
            (Some(Ordering::Less), _) => match self.next {
                Some(_) => SearchResult::Forward,
                None => SearchResult::End
            },
            (Some(Ordering::Greater), e) if e > 1 => SearchResult::Down,
            (_, _) => NotFound
        }
    }
    fn insert_to_node(node: Rc<RefCell<Node<K, V>>>, val: V) {
        node.borrow_mut().insert(val);
    }
    fn compare_nodes(left: Rc<RefCell<Node<K, V>>>, right: Rc<RefCell<Node<K, V>>>) -> Option<Ordering> {
        let right_key = &RefCell::borrow(&right).key;
        let left_key = &RefCell::borrow(&left).key;
        left_key.partial_cmp(right_key)
    }
    fn connect(left: Rc<RefCell<Node<K, V>>>, right: Rc<RefCell<Node<K, V>>>) {
        match Node::compare_nodes(left.clone(), right.clone()) {
            Some(Ordering::Less) => left.borrow_mut().insert_next(right.clone()),
            Some(Ordering::Greater) => right.borrow_mut().insert_next(left.clone()),
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
        match &self.head.borrow().next {
            Some(n) => self.search_in(n.clone(), key),
            _ => None
        }
    }

    pub fn insert(&mut self, key: K, val: V) -> Option<V> {
        if self.head.borrow().next.is_none() {
            let mut curr_node = Node::new_with(key.clone(), val.clone(), self.levels);
            &self.head.borrow_mut().upd_head(curr_node.clone());
            let mut cur_lvl = self.levels - 1;

            while cur_lvl > 0 {
                let under_node = Node::new_with(key.clone(), val.clone(), cur_lvl);
                RefCell::borrow_mut(&curr_node).set_under(Some(under_node.clone()));
                curr_node = under_node.clone();
                cur_lvl = cur_lvl - 1
            }
            None
        } else {
            let mut curr_node = self.head.borrow().next.as_ref().unwrap().clone();
            let mut prev_curr_node = curr_node.clone();
            let mut path_stack: Vec<Rc<RefCell<Node<K, V>>>> = vec![];
            loop {
                println!(">> before {:?}", curr_node.clone());
                let curr_node_clone = &curr_node.clone();
                let curr_node_b = RefCell::borrow(curr_node_clone);
                match curr_node_b.compare(&key) {
                    SearchResult::Forward => {
                        prev_curr_node = curr_node.clone();
                        curr_node = curr_node_b.next.as_ref().unwrap().clone();
                        println!(">> fwd {:?}", prev_curr_node.clone());
                        continue;
                    }
                    SearchResult::NotFound => {
                        let mut lower_node = Node::new_with(key.clone(), val.clone(), 1);

                        Node::connect(lower_node.clone(), curr_node.clone());

                        let mut curr_lvl: usize = 2;
                        let total_lvl = self.generator.random() + 1;
                        println!(">> nf {:?}", lower_node);
                        while curr_lvl <= total_lvl {
                            let next_node = Node::new_with(key.clone(), val.clone(),
                                                           curr_lvl);
                            next_node.borrow_mut().set_under(Some(lower_node.clone()));
                            let top_node = path_stack.pop().unwrap();

                            Node::connect(next_node.clone(), top_node.clone());

                            println!(">> nf {:?}", next_node);
                            lower_node = next_node.clone();
                            curr_lvl = curr_lvl + 1
                        }

                        return None;
                    }
                    SearchResult::Down => {
                        path_stack.push(prev_curr_node.clone());


                        if Node::is_under(prev_curr_node.clone()) {
                            curr_node = Node::get_under(prev_curr_node.clone());
                            println!(">> dwn {:?}", prev_curr_node.clone());
                            println!(" --- ");
                        } else {
                            return None;
                        }
                    }
                    SearchResult::End => {
                        path_stack.push(curr_node.clone());
                        match &curr_node_b.under {
                            Some(v) => curr_node = v.clone(),
                            _ => return None
                        }
                    }
                    SearchResult::Found(v) => {
                        Node::insert_to_node(curr_node, val.clone());
                        return Some(v);
                    }
                }
                prev_curr_node = curr_node.clone();
            }
        }
    }


    fn search_in(&self, node: Rc<RefCell<Node<K, V>>>, key: &K) -> Option<V> {
        let mut curr_node = node.clone();
        let mut prev_curr_node = curr_node.clone();
        loop {
            match RefCell::borrow(&curr_node.clone()).compare(key) {
                SearchResult::NotFound => return None,
                SearchResult::Forward => {
                    prev_curr_node = curr_node.clone();
                    curr_node = RefCell::borrow(&curr_node.clone()).next.as_ref().unwrap().clone();
                    continue;
                }
                SearchResult::Down =>
                    match &RefCell::borrow(&prev_curr_node.clone()).under {
                        Some(v) => curr_node = v.clone(),
                        _ => return None
                    },
                SearchResult::End => match &RefCell::borrow(&curr_node.clone()).under {
                    Some(v) => curr_node = v.clone(),
                    _ => return None
                },
                SearchResult::Found(v) => return Some(v),
            }
            prev_curr_node = curr_node.clone()
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
    fn simple_test() {
        let node = Node::new(10, 20, 3);
        assert_eq!(node.val, 20)
    }

    #[test]
    fn skip_list_test() {
        let mut list: SkipList<u64, u64> = SkipList::with_capacity(4000_000_000);
//        let _ = list.insert(20, 20);
        let _ = list.insert(20, 20);
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
        let mut gen = LevelGenerator::new(16);
        for _ in 0..100 {
            assert_eq!(true, gen.random() >= 0)
        }
    }
}

