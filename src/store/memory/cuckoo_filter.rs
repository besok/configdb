use std::marker::PhantomData;

struct CuckooFilter<T> {
    _mark: PhantomData<T>
}



#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}

