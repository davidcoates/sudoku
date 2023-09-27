use std::fmt;

#[derive(Clone,Copy,Eq,PartialEq,Debug)]
pub struct BitSet {
    bits: u128,
}

pub struct BitSetIter {
    bits: u128
}

impl BitSet {

    pub fn new() -> Self {
        BitSet{
            bits: 0,
        }
    }

    pub fn from_bits(bits: u128) -> Self {
        BitSet{
            bits: bits,
        }
    }

    pub fn single(value: usize) -> Self {
        if value >= 128 {
            panic!("value({}) out of bounds", value)
        }
        BitSet {
            bits: 1 << value,
        }
    }

    // [min, max]
    pub fn range(min: usize, max: usize) -> Self {
        if min > max || max >= 128 {
            panic!("range({}, {}) invalid", min, max);
        }
        let bits = ((1 << (max + 1)) - 1) & !((1 << min) - 1);
        return BitSet::from_bits(bits);
    }

    pub fn all() -> Self {
        BitSet{
            bits: u128::MAX,
        }
    }

    pub fn contains(&self, value: usize) -> bool {
        return (1 << value) & self.bits != 0;
    }

    pub fn iter(&self) -> BitSetIter {
        BitSetIter{ bits: self.bits }
    }

    pub fn value_unchecked(&self) -> usize {
        return self.iter().next().unwrap();
    }

    pub fn value(&self) -> Option<usize> {
        if self.len() == 1 { Some(self.value_unchecked()) } else { None }
    }

    pub fn min(&self) -> usize {
        if self.bits == 0 {
            panic!("min(0)")
        }
        return self.bits.trailing_zeros() as usize;
    }

    pub fn max(&self) -> usize {
        if self.bits == 0 {
            panic!("max(0)")
        }
        return self.iter().rev().next().unwrap();
    }

    pub fn empty(&self) -> bool { self.bits == 0 }

    pub fn len(&self) -> usize { self.bits.count_ones() as usize }

    pub fn insert(&mut self, value: usize) {
        if value >= 128 {
            panic!("value({}) out of bounds", value)
        }
        self.bits |= 1 << value;
    }

    pub fn remove(&mut self, value: usize) {
        if value >= 128 {
            panic!("value({}) out of bounds", value)
        }
        self.bits &= !(1 << value);
    }

    pub fn difference(&self, other: BitSet) -> BitSet {
        BitSet::from_bits(self.bits & !other.bits)
    }

    pub fn difference_with(&mut self, other: BitSet) {
        self.bits &= !other.bits;
    }

    pub fn intersection(&self, other: BitSet) -> BitSet {
        BitSet::from_bits(self.bits & other.bits)
    }

    pub fn intersect_with(&mut self, other: BitSet) {
        self.bits &= other.bits;
    }

    pub fn union(&self, other: BitSet) -> BitSet{
        BitSet::from_bits(self.bits | other.bits)
    }

    pub fn union_with(&mut self, other: BitSet) {
        self.bits |= other.bits;
    }

    pub fn complement(&self) -> BitSet {
        BitSet {
            bits: !self.bits,
        }
    }

}

impl fmt::Display for BitSet {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.len() == 0 {
            return write!(f, "<empty>");
        } else if self.len() == 1 {
            return write!(f, "{}", self.iter().next().unwrap());
        } else {
            return write!(f, "{}",  self.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
        }
    }

}

impl Iterator for BitSetIter {

    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        } else {
            let item = self.bits.trailing_zeros() as usize;
            self.bits ^= 1 << item;
            return Some(item);
        }
    }

}

impl DoubleEndedIterator for BitSetIter {

    fn next_back(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        } else {
            let item = self.bits.ilog2() as usize;
            self.bits ^= 1 << item;
            return Some(item);
        }
    }

}

pub trait Union<A = Self>: Sized {
    fn union<I>(iter: I) -> Self
        where I: Iterator<Item = A>;
}


impl Union for BitSet {
    fn union<I: Iterator<Item=Self>>(iter: I) -> Self {
        iter.fold(BitSet::new(), |a, b| a.union(b))
    }
}