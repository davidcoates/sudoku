pub type Digit = usize;

use std::fmt;
use crate::bit_set::*;

#[derive(Clone,Copy,Eq,PartialEq)]
pub struct Domain {
    bit_set: BitSet,
}

pub type Domains = Vec<Domain>;

impl fmt::Debug for Domain {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.solution() {
            Some(x) => write!(f, "{}", x),
            None => write!(f, "[{:?}]", self.bit_set)
        }
    }

}

impl Domain {

    pub fn new() -> Self {
        Domain {
            bit_set: BitSet::new(),
        }
    }

    pub fn all() -> Self {
        Domain {
            bit_set: BitSet::all(),
        }
    }

    pub fn insert(&mut self, digit: Digit) {
        self.bit_set.insert(digit);
    }

    pub fn len(&self) -> usize {
        return self.bit_set.len();
    }

    pub fn bit_set(&self) -> BitSet {
        return self.bit_set;
    }

    pub fn unsolvable(&self) -> bool {
        self.bit_set.len() == 0
    }

    pub fn solution(&self) -> Option<Digit> {
        if self.bit_set.len() == 1 {
            self.bit_set.iter().next()
        } else {
            None
        }
    }

    pub fn solved(&self) -> bool {
        self.bit_set.len() == 1
    }

    pub fn difference(&self, other: Domain) -> Domain {
        Domain { bit_set: self.bit_set.difference(other.bit_set) }
    }

    /*
    pub fn difference_with(&mut self, other: Domain) -> bool {
        let tmp = self.bit_set;
        self.bit_set.difference_with(other.bit_set);
        return tmp != self.bit_set;
    }

    pub fn intersection(&self, other: Domain) -> Domain {
        Domain { bit_set: self.bit_set.intersection(other.bit_set) }
    }
    */

    pub fn intersect_with(&mut self, other: Domain) -> bool {
        let tmp = self.bit_set.clone();
        self.bit_set.intersect_with(other.bit_set);
        return tmp != self.bit_set;
    }

    pub fn union_with(&mut self, other: Domain) -> bool {
        let tmp = self.bit_set.clone();
        self.bit_set.union_with(other.bit_set);
        return tmp != self.bit_set;
    }

}
