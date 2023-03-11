use std::fmt;

#[derive(Clone,Copy,Eq,PartialEq)]
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

    /*
    pub fn bits(&self) -> u128 {
        return self.bits;
    }
    */

    pub fn all() -> Self {
        BitSet{
            bits: u128::MAX,
        }
    }

    pub fn iter(&self) -> BitSetIter {
        BitSetIter{ bits: self.bits }
    }

/*
    pub fn combinations(&self) -> Vec<BitSetIter> {
        let mut ret = Vec::new();
        let values: Vec<usize> = self.iter().collect();
        for key in 0..u128::pow(2, values.len() as u32) {
            let mut bits: u128 = 0;
            for i in (BitSetIter { bits: key }) {
                bits |= 1 << values.get(i).unwrap();
            }
            ret.push(BitSetIter{ bits: bits });
        }
        return ret;
    }
*/
    pub fn empty(&self) -> bool { self.bits == 0 }

    pub fn len(&self) -> usize { self.bits.count_ones() as usize }

    pub fn insert(&mut self, value: usize) {
        if value >= 128 {
            panic!("value({}) out of bounds", value)
        }
        self.bits |= 1 << value;
    }

    pub fn difference(&self, other: BitSet) -> BitSet {
        BitSet::from_bits(self.bits & !other.bits)
    }

    /*
    pub fn difference_with(&mut self, other: BitSet) {
        self.bits &= !other.bits;
    }
    */

    pub fn intersection(&self, other: BitSet) -> BitSet {
        BitSet::from_bits(self.bits & other.bits)
    }

    pub fn intersect_with(&mut self, other: BitSet) {
        self.bits &= other.bits;
    }

    pub fn union_with(&mut self, other: BitSet) {
        self.bits |= other.bits;
    }

}

impl fmt::Debug for BitSet {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:b}", self.bits)
    }

}

impl Iterator for BitSetIter {

    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        } else {
            let item = self.bits.ilog2() as usize;
            self.bits ^= 1 << item;
            return Some(item);
        }
    }

}