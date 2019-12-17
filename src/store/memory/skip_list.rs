use std::rc::Rc;
use rand::distributions::{Uniform, Distribution};
use rand::prelude::ThreadRng;
use std::cmp::Ordering;
use crate::store::memory::skip_list::SearchResult::NotFound;
use std::cell::RefCell;

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
    fn total(&self) -> usize {
        self.total
    }
}

struct Head<K: Ord, V: Clone> {
    next: Option<Rc<RefCell<Node<K, V>>>>
}

impl<K: Ord, V: Clone> Head<K, V> {
    pub fn new(next: Option<Rc<RefCell<Node<K, V>>>>) -> Self {
        Head { next }
    }
    pub fn empty() -> Self {
        Head { next: None }
    }


    fn set_next(&mut self, node: Rc<RefCell<Node<K, V>>>) {

        if (&self).next.is_none(){
            self.next = Some(node)
        }else{
//            let key = &self.next.as_ref().unwrap().borrow_mut().key;
//            let in_key = &RefCell::borrow(&node).key;
//            let rc = RefCell::borrow(&self.next.as_ref().unwrap());
//            if let Some(Ordering::Greater) = key.partial_cmp(in_key) {
//              self.next = Some(node)
//            }
        }

    }
}


struct Node<K, V: Clone> {
    key: K,
    val: V,
    level: usize,
    prev: Option<Rc<RefCell<Node<K, V>>>>,
    next: Option<Rc<RefCell<Node<K, V>>>>,
    tower: Vec<Option<Rc<RefCell<Node<K, V>>>>>,
}

enum SearchResult<V: Clone> {
    Forward,
    NotFound,
    Down(usize),
    Found(V),
}

impl<K: Ord, V: Clone> Node<K, V> {
    fn new(key: K, val: V, level: usize) -> Self {
        let tower: Vec<Option<Rc<RefCell<Node<K, V>>>>> = vec![None; level];
        Node { key, val, level, tower, next: None, prev: None }
    }
    fn value(&self) -> V {
        self.val.clone()
    }

    fn compare(&self, key: &K) -> SearchResult<V> {
        match (self.key.partial_cmp(key), self.level) {
            (Some(Ordering::Equal), _) => SearchResult::Found(self.val.clone()),
            (Some(Ordering::Less), e) => match self.next {
                Some(_) => SearchResult::Forward,
                None => SearchResult::Down(e - 1)
            },
            (Some(Ordering::Greater), e) if e > 1 => SearchResult::Down(e - 1),
            (_, _) => NotFound
        }
    }
}

struct SkipList<K: Ord, V: Clone> {
    head: RefCell<Head<K, V>>,
    levels: usize,
    generator: LevelGenerator,
}

impl<K: Ord, V: Clone> SkipList<K, V> {
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
        let mut curr_node = self.head.borrow().next.as_ref().unwrap().clone();
        loop {
            match RefCell::borrow(&curr_node.clone()).compare(&key) {
                SearchResult::Forward => curr_node.replace_with(|n|n.next.as_ref().unwrap().clone()) ,
                SearchResult::Down(lvl) => match RefCell::borrow(&curr_node).tower.get(lvl) {
                    Some(Some(v)) => curr_node = v.clone(),
                    _ => return None
                },

                SearchResult::NotFound => return None,
                SearchResult::Found(v) => {
                    RefCell::borrow_mut(&curr_node).val = val;
                    return Some(v)
                },
            }
        }
    }


    fn search_in(&self, node: Rc<RefCell<Node<K, V>>>, key: &K) -> Option<V> {
        let mut curr_node = node;
        loop {
            match RefCell::borrow(&curr_node).compare(key) {
                SearchResult::NotFound => return None,
                SearchResult::Forward => curr_node = RefCell::borrow(&curr_node).next.as_ref().unwrap().clone(),
                SearchResult::Down(lvl) => match RefCell::borrow(&curr_node).tower.get(lvl) {
                    Some(Some(v)) => curr_node = v.clone(),
                    _ => return None
                },
                SearchResult::Found(v) => return Some(v),
            }
        }
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
    fn simple_skip_list_test() {
        let list: SkipList<u64, u64> = SkipList::with_capacity(4000_000_000);
        assert_eq!(list.levels, 31)
    }

    #[test]
    fn rand_test() {
        let mut gen = LevelGenerator::new(16);
        for _ in 0..100 {
            assert_eq!(true, gen.random() < gen.total());
            assert_eq!(true, gen.random() >= 0)
        }
    }
}

