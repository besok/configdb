use std::iter;

struct Node<K, V: Clone> {
    level: usize,
    tower: Vec<*mut Node<K, V>>,
    prev: Option<*mut Node<K, V>>,
    next: Option<*mut Node<K, V>>,

    key: K,
    val: V,
}

impl<K, V: Clone> Node<K, V> {
    fn new(key: K, val: V, level: usize) -> Self {
        Node {
            level,
            key,
            val,
            tower: Vec::with_capacity(level+1),
            prev: None,
            next: None,
        }
    }
    fn value(self) -> V {
        self.val.clone()
    }
}




#[cfg(test)]
mod tests {
    use crate::store::memory::skip_list::Node;

    #[test]
    fn simple_test() {
        let node = Node::new(10, 20, 3);
        let val = node.value();
        assert_eq!(val, 20)
    }
}

