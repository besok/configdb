trait Fingerprint<T> {
    fn fingerprint(self) -> Option<T>;
}

struct Polynomial {}
struct RabinFingerprint{}

impl Polynomial {
    fn newFromLong(value: i64) -> Self {}
    fn newFromBytes(value: Vec<u8>) -> Self {}
}

impl Fingerprint<Polynomial> for RabinFingerprint{
    fn fingerprint(self) -> Option<Polynomial> {
        unimplemented!()
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}