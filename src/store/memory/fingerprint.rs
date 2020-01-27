use lazy_static::lazy_static;

lazy_static! {
static ref Q:Polynomial = Polynomial::from_u64(2);
static ref X:Polynomial = Polynomial::from_u64(2);
static ref ONE:Polynomial = Polynomial::from_u64(1);
}



trait Fingerprint<T> {
    fn fingerprint(self) -> Option<T>;
}

struct Polynomial {
    degrees: Vec<u64>
}

enum Reducibility {
    REDUCIBLE,
    IRREDUCIBLE,
}

impl Polynomial {
    fn from_bytes(bytes: Vec<u8>, degree: u64) -> Self {
        Polynomial {
            degrees: {
                let mut vec: Vec<u64> =
                    (0..degree)
                        .filter(|el| check_bit(&bytes, el.clone() as usize))
                        .collect();
                vec.push(degree);
                vec.sort_by(|a, b| a.cmp(b).reverse());
                vec
            }
        }
    }

    fn from_u64(val: u64) -> Self {
        Polynomial {
            degrees: {
                let mut vec: Vec<u64> = (0..64)
                    .filter(|el| ((val >> el.clone()) & 1) == 1)
                    .collect();
                vec.sort_by(|a, b| a.cmp(b).reverse());
                vec
            }
        }
    }

    fn degree() -> u64{

    }
}

impl Clone for Polynomial{
    fn clone(&self) -> Self {
        Polynomial{
            degrees: self.degrees.clone()
        }
    }
}

fn check_bit_in(b: u8, idx: u8) -> bool {
    ((b >> idx) & 1) == 1
}

fn check_bit(bytes: &Vec<u8>, idx: usize) -> bool {
    let aidx = bytes.len() - 1 - (idx / 8);
    return
        bytes
            .get(aidx)
            .map(|b| check_bit_in(b.clone(), (idx % 8) as u8))
            .unwrap_or(false);
}

struct RabinFingerprint {}


#[cfg(test)]
mod test {
    use crate::store::memory::fingerprint::Polynomial;

    #[test]
    fn check_bit_test() {
        let p = Polynomial::from_bytes(vec![1, 2, 3, 4], 10);
        assert_eq!(p.degrees, vec![10, 9, 8, 2]);

        let p = Polynomial::from_u64(0x53);
        assert_eq!(p.degrees, vec![6, 4, 1, 0]);
        let p = Polynomial::from_u64(0x11B);
        assert_eq!(p.degrees, vec![8, 4, 3, 1, 0]);
    }
}