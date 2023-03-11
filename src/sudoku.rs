pub type Digit = usize;
type Index = usize;
type Variable = usize;

use std::rc::Rc;
use bit_set::BitSet;
use std::fmt;
use itertools::Itertools;

// TODO use inline memeory types for VariableSet and DomainSet
type VariableSet = BitSet;
type DomainSet = BitSet;
// type ConstraintSet = BitSet;

#[derive(Clone,Eq,PartialEq)]
pub struct Domain {
    pub bit_set: DomainSet,
}

impl fmt::Debug for Domain {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.bit_set)
    }

}

impl Domain {

    pub fn new() -> Self {
        return Domain {
            bit_set: DomainSet::new(),
        }
    }

    pub fn singleton(digit: Digit) -> Self {
        let mut domain = Domain::new();
        domain.bit_set.insert(digit);
        return domain;
    }

    pub fn sudoku() -> Self {
        let mut domain = Domain::new();
        for digit in 1..10 {
            domain.bit_set.insert(digit);
        }
        return domain;
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

}

pub fn mux(x: Index, y: Index) -> Variable {
    (x - 1) * 9 + (y - 1)
}

pub fn demux(xy: Variable) -> (Index, Index) {
    let x = xy / 9;
    let y = xy - 9 * x;
    (x + 1, y + 1)
}

#[derive(Debug,Clone,Copy)]
pub enum Result {
    Satisfied,
    Unsatisfiable,
    Progress,
    Stuck
}

pub trait Constraint: fmt::Debug {

    fn simplify(&self, domains: &mut Domains) -> Result;

    fn variables(&self) -> &VariableSet;

}

#[derive(Debug,Clone)]
pub struct BoxedConstraint {
    constraint: Rc<dyn Constraint>,
    result: Option<Result>
}

fn check(variables: &VariableSet, domains: &Domains) -> Result {
    let mut solved = true;
    for variable in variables {
        let domain = domains.get(variable).unwrap();
        if domain.unsolvable() {
            return Result::Unsatisfiable;
        }
        if !domain.solved() {
            solved = false;
        }
    }
    return if solved { Result::Satisfied } else  { Result::Stuck };
}

impl BoxedConstraint {

    pub fn new(constraint: Rc<dyn Constraint>) -> Self {
        return BoxedConstraint {
            constraint,
            result: None,
        }
    }

    pub fn simplify(&mut self, domains: &mut Domains) -> Result {
        match self.result {
            Some(result) => result,
            None => {
                let result = self.constraint.as_ref().simplify(domains);
                match result {
                    Result::Satisfied => { self.result = Some(Result::Satisfied); return Result::Satisfied; }
                    Result::Unsatisfiable => { self.result = Some(Result::Unsatisfiable); return Result::Unsatisfiable; }
                    _ => { return result; }
                }
            }
        }
    }

    pub fn constraint(&self) -> &dyn Constraint {
        return self.constraint.as_ref();
    }

}

// Distinct digits in range
pub struct DistinctN {
    min: Digit,
    max: Digit,
    domain: DomainSet,
    variables: VariableSet,
}

impl DistinctN {

    pub fn new(min: Digit, max: Digit, variables: VariableSet) -> DistinctN {
        if min >= max || max - min + 1 != variables.len() {
            panic!("bad DistinctN: {:?}", variables)
        }
        let mut domain = DomainSet::new();
        for i in min..max+1 {
            domain.insert(i);
        }
        return DistinctN {
            min,
            max,
            domain,
            variables,
        };
    }
}

impl fmt::Debug for DistinctN {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{},{}] : {}",
            self.min,
            self.max,
            self.variables.iter().map(|x| format!("{:?}", demux(x))).format(" "))
    }

}

impl Constraint for DistinctN {

    fn simplify(&self, domains: &mut Domains) -> Result {
        for variable in &self.variables {
            let domain = domains.get(variable).unwrap();
            if domain.unsolvable() {

            }
            if domain.solved() {

            }
        }
        /*
        let mut unseen = self.domain.clone();
        for variable in &self.variables {
            if variable == target_variable {
                continue;
            }
            unseen.difference_with(&domains.get(variable).unwrap().bit_set);
        }
        print!("{:?} {:?}", target_domain, unseen);
        target_domain.bit_set.intersect_with(&unseen);
        */
        Result::Stuck
    }

    fn variables(&self) -> &VariableSet {
        &self.variables
    }

}

type Domains = Vec<Domain>;
type Constraints = Vec<BoxedConstraint>;

pub struct State {
    pub domains: Domains,
    pub constraints: Constraints,
}

impl fmt::Display for State {

    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (1..10).map(|i|{
            writeln!(f, "{:?}", (1..10).map(|j| self.domains.get(mux(i, j)).unwrap()).format(" "))
        }).collect()
    }

}

impl State {

    pub fn sudoku(digits: &[[Digit; 9]; 9]) -> Self {
        let mut domains = Domains::new();
        for i in 0..9 {
            for j in 0..9 {
                let digit = digits[i][j];
                if digit == 0 {
                    domains.push(Domain::sudoku());
                } else {
                    domains.push(Domain::singleton(digit));
                }
            }
        }
        let mut constraints = Constraints::new();
        for i in 1..10 {
            let mut variables = VariableSet::new();
            for j in 1..10 {
                variables.insert(mux(i, j));
            }
            constraints.push(BoxedConstraint::new(Rc::new(DistinctN::new(1, 9, variables))));
        }
        for i in 1..10 {
            let mut variables = VariableSet::new();
            for j in 1..10 {
                variables.insert(mux(j, i));
            }
            constraints.push(BoxedConstraint::new(Rc::new(DistinctN::new(1, 9, variables))));
        }
        for x in 0..3 {
            for y in 0..3 {
                let mut variables = VariableSet::new();
                for i in 1..4 {
                    for j in 1..4 {
                        variables.insert(mux(x*3 + i, y*3 + j));
                    }
                }
                constraints.push(BoxedConstraint::new(Rc::new(DistinctN::new(1, 9, variables))));
            }
        }
        return State{
            domains,
            constraints,
        };
    }

/*
    pub fn solved(&self) -> bool {
        for domain in &self.domains {
            if !domain.solved() {
                return false;
            }
        }
        return true;
    }

    pub fn unsolvable(&self) -> bool {
        for domain in &self.domains {
            if domain.unsolvable() {
                return true;
            }
        }
        return false;
    }


    fn check(&self) -> Result {
        let mut solved = true;
        for domain in &self.domains {
            if domain.unsolvable() {
                return Result::Unsolvable;
            }
            if !domain.solved() {
                solved = false;
            }
        }
        if solved {
            return Result::Solved;
        }
        return Result::Stuck;
    }
*/

    // TODO optimise this for:
    // order of constraints
    // only constraints with dirty variable
    pub fn simplify(&mut self) -> Result {
        let mut progress = false;
        for constraint in &mut self.constraints {
            let result = constraint.simplify(&mut self.domains);
            match result {
                Result::Unsatisfiable => { return Result::Unsatisfiable; }
                Result::Progress => { progress = true; }
                _ => { }
            }
        }
        return if progress { Result::Progress } else { Result::Stuck };
    }

} 