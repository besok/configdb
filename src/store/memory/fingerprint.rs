trait Fingerprint<T> {
    fn fingerprint(self) -> Option<T>;
}

struct Polynomial {}
struct RabinFingerprint{}


#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}