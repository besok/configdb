use lazy_static::lazy_static;
use crate::store::memory::fingerprint::Reducibility::{REDUCIBLE, IRREDUCIBLE};
use std::cmp::Ordering;

lazy_static! {
static ref X:Polynomial = Polynomial::from_u64(2);
static ref ONE:Polynomial = Polynomial::from_u64(1);
}



trait Fingerprint<T> {
    fn fingerprint(self) -> Option<T>;
}

struct Polynomial {
    degrees: Vec<i64>
}

enum Reducibility {
    REDUCIBLE,
    IRREDUCIBLE,
}

impl PartialEq for Polynomial {
    fn eq(&self, other: &Self) -> bool {
        self.degrees.eq(&other.degrees)
    }
}

impl PartialOrd for Polynomial {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            match self.degree().cmp(&other.degree()) {
                Ordering::Equal => {
                    match Polynomial::xor(self.clone(), other.clone()) {
                        Polynomial { degrees } if degrees.is_empty() => Ordering::Equal,
                        p @ _ =>
                            if self.degrees.contains(&p.degree()) {
                                Ordering::Greater
                            } else { Ordering::Less }
                    }
                }
                r @ _ => r
            })
    }
}

impl Polynomial {
    fn from_bytes(bytes: Vec<u8>, degree: i64) -> Self {
        Polynomial {
            degrees: {
                let mut vec: Vec<i64> =
                    (0..degree)
                        .filter(|el| check_bit(&bytes, el.clone() as usize))
                        .collect();
                vec.push(degree);
                vec.sort_by(|a, b| a.cmp(b).reverse());
                vec.dedup_by(|a, b| a == b);
                vec
            }
        }
    }
    fn from_degrees(degrees: Vec<i64>) -> Self {
        Polynomial {
            degrees: {
                let mut vec = degrees.clone();
                vec.sort_by(|a, b| a.cmp(b).reverse());
                vec.dedup_by(|a, b| a == b);
                vec
            }
        }
    }
    fn from_u64(val: i64) -> Self {
        Polynomial {
            degrees: {
                let mut vec: Vec<i64> = (0..64)
                    .filter(|el| ((val >> el.clone()) & 1) == 1)
                    .collect();
                vec.sort_by(|a, b| a.cmp(b).reverse());
                vec.dedup_by(|a, b| a == b);
                vec
            }
        }
    }

    fn degree(&self) -> i64 {
        match self.degrees.first() {
            None => -1,
            Some(el) => el.clone() as i64
        }
    }

    fn degrees(&self) -> Vec<i64> {
        self.degrees.clone()
    }

    fn add(&self, p: Polynomial) -> Self {
        Polynomial::xor(self.clone(), p)
    }
    fn subtract(&self, p: Polynomial) -> Self {
        Polynomial::xor(self.clone(), p)
    }
    fn multiply(&self, p: Polynomial) -> Self {
        let mut degrees: Vec<i64> = vec![];
        for l in self.degrees() {
            for r in p.degrees() {
                let s = l + r;
                if degrees.contains(&s) {
                    let idx = degrees.iter().position(|x| *x == s).unwrap();
                    degrees.remove(idx);
                } else {
                    degrees.push(s)
                }
            }
        }
        Polynomial { degrees }
    }

    fn and(left_p: Polynomial, right_p: Polynomial) -> Self {
        Polynomial {
            degrees: { vec_retain_all(left_p.degrees(), right_p.degrees()) }
        }
    }
    fn or(left_p: Polynomial, right_p: Polynomial) -> Self {
        Polynomial {
            degrees: { vec_add_all(left_p.degrees(), right_p.degrees()) }
        }
    }

    fn mod_op(left_p: Polynomial, right_p: Polynomial) -> Self {
        let da = left_p.degree();
        let db = right_p.degree();
        let mut register = left_p.clone();
        let mut i = da - db;
        while i >= 0 {
            let x = i + db;
            if register.degrees.contains(&x) {
                register = Polynomial::xor(register.clone(),
                                           Polynomial::shift_left(right_p.clone(), i))
            }
            i -= 1
        }
        register
    }

    fn shift_left(p: Polynomial, shift: i64) -> Self {
        let mut degrees: Vec<i64> = vec![];
        for el in p.degrees() {
            degrees.push(el + shift)
        }
        Polynomial::from_degrees(degrees)
    }

    fn xor(left_p: Polynomial, right_p: Polynomial) -> Self {
        let left = vec_rem_all(left_p.degrees(), right_p.degrees());
        let right = vec_rem_all(right_p.degrees(), left_p.degrees());

        let right = vec_rem_all(right, left_p.degrees());

        let right = vec_add_all(right, left);

        Polynomial { degrees: right }
    }
    fn reducibility(&self) -> Reducibility {
        let d = self.degree();
        for el in 1..=d / 2 {}

        IRREDUCIBLE
    }

    fn reduce_exp(v: i64) -> Self {
        let two: i64 = 2;
        let x = two.pow(v as u32);
        let mut bc = v;
        while bc.count_ones() != 0 {

        }

        Polynomial {
            degrees: vec![]
        }
    }
//    fn mod_pow(l: Polynomial, r: Polynomial, e: i64) -> Self {
//        let mut r = ONE.clone();
//        let a = l.clone();
//    }
    fn gcd(p_left: Polynomial, p_right: Polynomial) -> Self {
        let mut a = p_left.clone();
        let mut b = p_right.clone();
        while !b.degrees.is_empty() {
            let b_p = b.clone();
            b = Polynomial::mod_op(a.clone(), b.clone());
            a = b_p;
        }
        return a.clone();
    }
}

impl Clone for Polynomial {
    fn clone(&self) -> Self {
        Polynomial {
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

fn vec_rem_all<T: Ord + Clone>(src: Vec<T>, dst: Vec<T>) -> Vec<T> {
    let mut loc_src = src.clone();
    loc_src.retain(|el| !dst.contains(el));
    loc_src
}

fn vec_add_all<T: Ord + Clone>(src: Vec<T>, dst: Vec<T>) -> Vec<T> {
    let mut src_loc = [&src[..], &dst[..]].concat();
    src_loc.sort_by(|a, b| a.cmp(b).reverse());
    src_loc.dedup_by(|a, b| a == b);
    src_loc
}

fn vec_retain_all<T: Ord + Clone>(src: Vec<T>, dst: Vec<T>) -> Vec<T> {
    let mut loc_src = src.clone();
    loc_src.retain(|el| dst.contains(el));
    loc_src
}

struct RabinFingerprint {}


#[cfg(test)]
mod test {
    use crate::store::memory::fingerprint::{Polynomial, vec_rem_all};

    #[test]
    fn s_test(){
        assert_eq!(10i64.count_ones(),2);
        assert_eq!(100i64.count_ones(),3);
        assert_eq!(100100i64.count_ones(),6);
        assert_eq!(1000i64.count_ones(),6);
    }

    #[test]
    fn xor_test() {
        let left = Polynomial::from_u64(100123);
        let right = Polynomial::from_u64(123100);
        let res = Polynomial::xor(left.clone(), right.clone());
        assert_eq!(res.degrees, vec![14, 13, 10, 9, 8, 7, 6, 2, 1, 0]);
        let res = Polynomial::xor(right.clone(), left.clone());
        assert_eq!(res.degrees, vec![14, 13, 10, 9, 8, 7, 6, 2, 1, 0]);
        let res = Polynomial::xor(left.clone(), left.clone());
        assert_eq!(res.degrees, vec![])
    }

    #[test]
    fn rem_all_test() {
        let vec1 = vec![1, 2, 3, 4, 5];
        let vec2 = vec![1, 2, 3];

        assert_eq!(vec_rem_all(vec1.clone(), vec2.clone()), vec![4, 5]);
        assert_eq!(vec_rem_all(vec2.clone(), vec1.clone()), vec![])
    }

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