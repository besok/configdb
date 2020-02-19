//! Simple implementation for rabin fingerprint [wiki](https://en.wikipedia.org/wiki/Rabin_fingerprint)
//! The base entity is polynomial.
//! 2 major implementation:
//! - rabin fingerprint (default)
//! - fix rabin fingerpint (uses i64 and lookup tables to increase performance.)
use crate::store::structures::fingerprint::Reducibility::{REDUCIBLE, IRREDUCIBLE};
use std::cmp::Ordering;
use rand::{Rng};

pub struct FixRabinFingerprint {
    shift: i64,
    degree: i64,
    table: [i64; 512],
}

pub struct RabinFingerprint {
    p: Polynomial,
    base: Polynomial,
}

pub struct Polynomial {
    degrees: Vec<i64>
}

pub trait Fingerprint<T> {
    fn calculate(&mut self, bytes: Vec<u8>) -> Option<T>;
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
            }
        )
    }
}

impl Polynomial {
    pub fn from_u64(val: i64) -> Self {
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
    pub fn from_degree_irr(d: i32) -> Self {
        loop {
            let p = Polynomial::from_random(d);
            if let IRREDUCIBLE = p.reducibility() {
                return p;
            }
        }
    }
    pub fn from_bytes(bytes: Vec<u8>, degree: i64) -> Self {
        Polynomial {
            degrees: {
                let mut vec: Vec<i64> = (0..degree)
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
    fn empty() -> Self {
        Polynomial { degrees: vec![] }
    }
    fn from_random(d: i32) -> Polynomial {
        let r = d / 8 + 1;
        let mut v = Vec::with_capacity(r as usize);

        for _ in 0..r {
            let random_number: u8 = rand::thread_rng().gen();
            v.push(random_number)
        }

        Polynomial::from_bytes(v, d as i64)
    }

}

impl Polynomial {
    pub fn to_i64(&self) -> i64 {
        let mut b = 0;
        for el in self.degrees() {
            b = b | (1 << el)
        }
        b
    }

    fn degree(&self) -> i64 {
        match self.degrees.first() {
            None => -1,
            Some(el) => el.clone()
        }
    }
    fn degrees(&self) -> Vec<i64> {
        self.degrees.clone()
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
    fn or(&self, right_p: Polynomial) -> Self {
        Polynomial {
            degrees: { vec_add_all(self.degrees(), right_p.degrees()) }
        }
    }

    fn modulo(&self, p: Polynomial) -> Self {
        let da = self.degree();
        let db = p.degree();
        let mut register = self.clone();
        let mut i = da - db;
        while i >= 0 {
            let x = i + db;
            if register.degrees.contains(&x) {
                register = Polynomial::xor(register.clone(), p.clone().shift_left(i))
            }
            i -= 1
        }
        register
    }

    fn shift_left(&self, shift: i64) -> Self {
        let mut degrees: Vec<i64> = vec![];
        for el in self.degrees() {
            degrees.push(el + shift)
        }
        Polynomial::from_degrees(degrees)
    }

    fn xor(left_p: Polynomial, right_p: Polynomial) -> Self {
        let left = vec_rem_all(left_p.degrees(), right_p.degrees());
        let right = vec_rem_all(right_p.degrees(), left_p.degrees());
        let degrees = vec_add_all(right, left);

        Polynomial { degrees }
    }
    fn reducibility(&self) -> Reducibility {
        let one = Polynomial::from_u64(1);
        let two = Polynomial::from_u64(2);

        if let Some(Ordering::Equal) = self.partial_cmp(&one) {
            return REDUCIBLE;
        }
        if let Some(Ordering::Equal) = self.partial_cmp(&two) {
            return REDUCIBLE;
        }

        for el in 1..=self.degree() / 2 {
            let b = self.reduce_exp(el);
            let g = Polynomial::gcd(self.clone(), b);
            match g.partial_cmp(&one) {
                Some(Ordering::Less) | Some(Ordering::Greater) => return REDUCIBLE,
                _ => ()
            }
        }

        IRREDUCIBLE
    }

    fn reduce_exp(&self, v: i64) -> Self {
        let two: i64 = 2;
        let x = two.pow(v as u32);
        let p_x = Polynomial::from_u64(2);
        let p_m = Polynomial::modulo_pow(p_x.clone(), self.clone(), x);
        let p = Polynomial::xor(p_m, p_x.clone());
        p.modulo(self.clone())
    }
    fn modulo_pow(l: Polynomial, r: Polynomial, e: i64) -> Self {
        let mut res = Polynomial::from_u64(1);
        let mut b = l.clone();
        let mut e = e;

        while e.count_ones() != 0 {
            if e & (1 << 0) != 0 {
                res = res.multiply(b.clone()).modulo(r.clone())
            }
            e = e >> 1;
            b = b.multiply(b.clone()).modulo(r.clone())
        }

        res
    }
    fn gcd(p_left: Polynomial, p_right: Polynomial) -> Self {
        let mut a = p_left.clone();
        let mut b = p_right.clone();
        while !b.degrees.is_empty() {
            let b_p = b.clone();
            b = a.clone().modulo(b.clone());
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

fn check_bit(bytes: &Vec<u8>, idx: usize) -> bool {
    let aidx = bytes.len() - 1 - (idx / 8);
    return
        bytes
            .get(aidx)
            .map(|b| ((b.clone() >> (idx % 8) as u8) & 1) == 1)
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


impl Fingerprint<i64> for RabinFingerprint {
    fn calculate(&mut self, bytes: Vec<u8>) -> Option<i64> {
        <RabinFingerprint as Fingerprint<Polynomial>>::calculate(self, bytes).map(|p| p.to_i64())
    }
}

impl Fingerprint<Polynomial> for RabinFingerprint {
    fn calculate(&mut self, bytes: Vec<u8>) -> Option<Polynomial> {
        for el in bytes {
            self.push_byte(el)
        }
        Some(self.return_then_clean())
    }
}

impl RabinFingerprint {
    pub fn new(base: Polynomial) -> Self {
        RabinFingerprint { p: Polynomial::empty(), base }
    }
    pub fn default() -> Self {
        RabinFingerprint::new(Polynomial::from_degree_irr(53))
    }

    fn push_byte(&mut self, byte: u8) {
        self.p = self.p.clone()
            .shift_left(8)
            .or(Polynomial::from_u64((byte & 0xFF) as i64))
            .modulo(self.base.clone());
    }

    fn return_then_clean(&mut self) -> Polynomial{
        let p = self.p.clone();
        self.p = Polynomial::empty();
        p
    }
}

impl FixRabinFingerprint {
    pub fn new(base: Polynomial) -> Self {
        let degree = base.degree();
        let shift = degree - 8;
        let table = {
            let mut table = [0; 512];
            for el in 0..512 {
                let left = Polynomial::from_u64(el).shift_left(degree);
                let md = left.modulo(base.clone());
                let res = Polynomial::xor(left, md);
                table[el as usize] = res.to_i64()
            }
            table
        };

        FixRabinFingerprint {
            degree,
            shift,
            table,
        }
    }
    pub fn new_degree(d: i32) -> Self {
        FixRabinFingerprint::new(Polynomial::from_degree_irr(d))
    }
}

impl Fingerprint<i64> for FixRabinFingerprint {
    fn calculate(&mut self, bytes: Vec<u8>) -> Option<i64> {
        Some({
            let mut f = 0;
            for b in bytes {
                let x = (f >> self.shift) & 0x1FF;
                f = ((f << 8) | (b & 0xFF) as i64) ^ self.table[x as usize]
            }
            f
        })
    }
}

#[cfg(test)]
mod test {
    use crate::store::structures::fingerprint::{Polynomial, vec_rem_all, RabinFingerprint, Fingerprint, FixRabinFingerprint};
    use crate::store::structures::fingerprint::Reducibility::IRREDUCIBLE;

    #[test]
    fn fingerprint_test() {
        let mut f = FixRabinFingerprint::new_degree(53);
        for _ in 1..1000 {
            let rand = f.calculate(vec![1, 1, 10, 0, 127]);
            assert_eq!(Some(4312399999), rand)
        }
    }

    #[test]
    fn reduce_test() {
        let n = Polynomial { degrees: vec![3, 1, 0] };

        let one = Polynomial { degrees: vec![1] };

        let res = Polynomial::modulo_pow(one, n.clone(), 2);
        assert_eq!(res.degrees, vec![2]);

        let next = n.reduce_exp(1);
        assert_eq!(next.degrees, vec![2, 1])
    }

    #[test]
    fn mod_test() {
        let n = Polynomial { degrees: vec![7, 5, 4, 2, 1, 0] };
        let res = n.to_i64();
        assert_eq!(res, 183);
        let o = Polynomial { degrees: vec![2, 1] };
        let res = Polynomial::modulo_pow(o.clone(), n.clone(), 2);
        assert_eq!(res.degrees, vec![4, 2])
    }

    #[test]
    fn irr_test() {
        let p = Polynomial {
            degrees: vec![3, 1, 0]
        };

        if let IRREDUCIBLE = p.reducibility() {} else {
            panic!(" irr ")
        }
    }

    #[test]
    fn time_test() {
        let mut fpr = RabinFingerprint::default();
        for el in 1..10000 {
            let p: i64 = fpr.calculate(vec![1, 2, 3]).unwrap();
        }
    }

    #[test]
    fn s_test() {
        let base = Polynomial { degrees: vec![7, 3, 0] };
        let mut f = RabinFingerprint::new(base);


        let p: i64 = f.calculate(vec![1, 1, 10, 0, 127]).unwrap();
        let dgr = f.p.degrees;
        assert_eq!(dgr, vec![5, 4, 1]);
        assert_eq!(p, 50)
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
    fn check_idempotent_test() {
        let base = Polynomial { degrees: vec![7, 3, 0] };
        let mut f = RabinFingerprint::new(base);

        let res: i64 = f.calculate(vec![1, 2, 3]).unwrap();
        assert_eq!(res, 49);
        let res: i64 = f.calculate(vec![1, 2, 3]).unwrap();
        assert_eq!(res, 49);
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