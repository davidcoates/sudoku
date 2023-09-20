use std::rc::Rc;

use crate::types::*;
use crate::bit_set::*;

pub trait Tracker {

    fn variable_name(&self, variable: Variable) -> &String;

    fn on_progress(&mut self, comment: String);
}

pub enum Result {
    Unsolvable,
    Solved,
    Stuck,
    Progress(Vec<BoxedConstraint>)
}

pub trait Constraint {

    fn variables(&self) -> &VariableSet;

    fn simplify(self: Rc<Self>, domains: &mut Domains, tracker: &mut dyn Tracker) -> Result;

    fn description(&self) -> &String;
}

pub type Constraints = Vec<BoxedConstraint>;

#[derive(Clone)]
pub struct BoxedConstraint {
    constraint: Rc<dyn Constraint>,
}

// TODO add caching of domains to detect if any relevant domain has changed.

impl BoxedConstraint {

    pub fn new(constraint: Rc<dyn Constraint>) -> Self {
        return BoxedConstraint {
            constraint,
        }
    }

    pub fn unbox(&self) -> &dyn Constraint {
        return self.constraint.as_ref();
    }

    fn check(&self, domains: &mut Domains) -> Option<Result> {
        let mut all_solved = true;
        for variable in self.unbox().variables().iter() {
            if domains.get(variable).unwrap().len() == 0 {
                return Some(Result::Unsolvable);
            } else if domains.get(variable).unwrap().len() != 1 {
                all_solved = false;
            }
        }
        return if all_solved { Some(Result::Solved) } else { None };
    }

    pub fn simplify(&self, domains: &mut Domains, tracker: &mut dyn Tracker) -> Result {
        match self.check(domains) {
            Some(result) => result,
            None => self.constraint.clone().simplify(domains, tracker),
        }
    }

}

fn progress_simplify(constraint: BoxedConstraint, domains: &mut Domains, tracker: &mut dyn Tracker) -> Result {
    match constraint.simplify(domains, tracker) {
        Result::Stuck => {
            return Result::Progress(vec![constraint]);
        },
        r => r
    }
}

fn join(c1: BoxedConstraint, c2: BoxedConstraint, domains: &mut Domains, tracker: &mut dyn Tracker) -> Result {
    let mut r1 = progress_simplify(c1, domains, tracker);
    let mut r2 = progress_simplify(c2, domains, tracker);
    match (&r1, &r2) {
        (Result::Unsolvable, _)       => Result::Unsolvable,
        (_, Result::Unsolvable)       => Result::Unsolvable,
        (Result::Solved, Result::Solved) => Result::Solved,
        (Result::Stuck, Result::Stuck)   => Result::Stuck,
        _                              => {
            let mut tmp = Vec::new();
            match &mut r1 {
                Result::Progress(constraints) => { tmp.append(constraints); },
                _ => {}
            }
            match &mut r2 {
                Result::Progress(constraints) => { tmp.append(constraints); },
                _ => {}
            }
            return Result::Progress(tmp);
        }
    }
}

// Distinct digits covering a domain
#[derive(Clone)]
pub struct Permutation {
    description: String,
    variables: VariableSet,
    domain: Domain,
}

impl Permutation {

    pub fn new(description: String, variables: VariableSet, domain: Domain) -> Self {
        if variables.len() != domain.len() {
            panic!("bad Permutation: #variables != #domain")
        }
        return Permutation {
            description,
            variables,
            domain
        };
    }

}

impl Constraint for Permutation {

    fn simplify(self: Rc<Self>, domains: &mut Domains, tracker: &mut dyn Tracker) -> Result {

        let mut progress = false;

        // Firstly, intersect against this constraint's domain
        for variable in self.variables.iter() {
            let new = domains.get_mut(variable).unwrap();
            let old = *new;
            new.intersect_with(self.domain);
            if *new != old {
                progress = true;
                tracker.on_progress(format!("{} is not {} since {}", tracker.variable_name(variable), old.difference(*new), self.description));
            }
        }

        // Solve based on the following idea:
        //
        // Suppose there is a set S of n cells, and the union of domains of S is I.
        // If:
        //  * I has length n
        // Then:
        //  * The domain of each cell not in S can be subtracted by I.
        //
        // E.g. if there are three cells A = (12) and B = (23), and C = (13), then (123) can be removed from all other cells.

        let values: Vec<usize> = self.variables.iter().collect();
        for combination in 1..(u128::pow(2, values.len() as u32) - 1) {
            let mut selection = BitSet::new();
            let mut union = Domain::new();
            for i in BitSet::from_bits(combination).iter() {
                let variable = *values.get(i).unwrap();
                selection.insert(variable);
                union.union_with(*domains.get(variable).unwrap());
            }
            if union.len() == selection.len() {
                let reason1 = if union.len() == 1 {
                    format!("{}", self.description)
                } else {
                    format!("is inside {} tuple", union)
                };
                let c1 = BoxedConstraint::new(Rc::new(Permutation::new(reason1, selection, union)));
                let selection_complement = self.variables.difference(selection);
                let reason2 = if union.len() == 1 {
                    format!("{}", self.description)
                } else {
                    format!("is outside {} tuple", union)
                };
                let c2 = BoxedConstraint::new(Rc::new(Permutation::new(reason2, selection_complement, self.domain.difference(union))));
                return join(c1, c2, domains, tracker);
            }
        }


        // Solve based on the following idea:
        //
        // Suppose there is a set S of n cells, and the intersection of domains of S is I.
        // If:
        //  * I does not overlap with the domain of any cell not in S, and
        //  * I has length n
        // Then:
        //  * The domain of each cell in S can be intersected with I
        //  * The domain of each cell not in S can be subtracted by I.
        //
        // E.g. if there are two cells A = (123) and B = (124), and no other cell contains a 1 or a 2,
        // then A = B = (12), and both 1 and 2 are removed from the domain of every remaining cell.

        let values: Vec<usize> = self.variables.iter().collect();
        for combination in 1..(u128::pow(2, values.len() as u32) - 1) {
            let mut selection = Domain::new();
            let mut intersection = Domain::all();
            for i in Domain::from_bits(combination).iter() {
                let variable = *values.get(i).unwrap();
                selection.insert(variable);
                intersection.intersect_with(*domains.get(variable).unwrap());
            }
            if intersection.len() == selection.len() {
                let mut ok = true;
                let selection_complement = self.variables.difference(selection);
                for variable in selection_complement.iter() {
                    if !domains.get(variable).unwrap().intersection(intersection).empty() {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    let reason1 = format!("inside {} disguised tuple", intersection);
                    let c1 = BoxedConstraint::new(Rc::new(Permutation::new(reason1, selection, intersection)));
                    let reason2 = format!("outside {} disguised tuple", intersection);
                    let c2 = BoxedConstraint::new(Rc::new(Permutation::new(reason2, selection_complement, self.domain.difference(intersection))));
                    return join(c1, c2, domains, tracker);
                }
            }
        }

        if progress {
            return Result::Progress(vec![BoxedConstraint::new(self)]);
        } else {
            return Result::Stuck;
        }
    }

    fn variables(&self) -> &VariableSet {
        &self.variables
    }

    fn description(&self) -> &String {
        return &self.description
    }
}


// Strictly increasing digits
#[derive(Clone)]
pub struct Increasing {
    description: String,
    variables: Vec<Variable>,
    variable_set: VariableSet,
}

impl Increasing {

    pub fn new(description: String, variables: Vec<Variable>) -> Self {
        let mut variable_set = VariableSet::new();
        for variable in variables.iter() {
            variable_set.insert(*variable);
        }
        return Increasing {
            description,
            variables,
            variable_set,
        };
    }

}

impl Constraint for Increasing {

    fn simplify(self: Rc<Self>, domains: &mut Domains, tracker: &mut dyn Tracker) -> Result {

        let mut progress = false;

        // Restrict small values
        let mut min : Option<usize> = None;
        for variable in self.variables.iter() {
            match min {
                Some(n) => {
                    let new = domains.get_mut(*variable).unwrap();
                    let old = *new;
                    new.difference_with(Domain::range(0, n));
                    if *new != old {
                        progress = true;
                        tracker.on_progress(format!("{} is not {} considering increasing min of {}", tracker.variable_name(*variable), old.difference(*new), self.description));
                    }
                }
                _ => {}
            }

            min = Some(domains.get(*variable).unwrap().min());
        }

        // Restrict large values
        let mut max : Option<usize> = None;
        for variable in self.variables.iter().rev() {
            match max {
                Some(n) => {
                    let new = domains.get_mut(*variable).unwrap();
                    let old = *new;
                    new.intersect_with(Domain::range(0, n - 1));
                    if *new != old {
                        progress = true;
                        tracker.on_progress(format!("{} is not {} considering decreasing max of {}", tracker.variable_name(*variable), old.difference(*new), self.description));
                    }
                }
                _ => {}
            }

            max = Some(domains.get(*variable).unwrap().max());
        }

        if progress {
            return Result::Progress(vec![BoxedConstraint::new(self)]);
        } else {
            return Result::Stuck;
        }
    }

    fn variables(&self) -> &VariableSet {
        &self.variable_set
    }

    fn description(&self) -> &String {
        return &self.description;
    }
}

#[derive(Clone)]
pub struct Equals {
    description: String,
    variables: VariableSet,
}

impl Equals {

    pub fn new(description: String, variables: VariableSet) -> Self {
        return Equals {
            description,
            variables,
        };
    }

}

impl Constraint for Equals {

    fn simplify(self: Rc<Self>, domains: &mut Domains, tracker: &mut dyn Tracker) -> Result {

        // Compute the intersection of all domains
        let mut domain = Domain::all();
        for variable in self.variables.iter() {
            domain.intersect_with(*domains.get_mut(variable).unwrap());
        }

        let mut progress = false;
        for variable in self.variables.iter() {
            let new = domains.get_mut(variable).unwrap();
            let old = *new;
            new.intersect_with(domain);
            if *new != old {
                progress = true;
                tracker.on_progress(format!("{} is not {} since {}", tracker.variable_name(variable), old.difference(*new), self.description));
            }
        }

        if progress {
            return Result::Progress(vec![BoxedConstraint::new(self)]);
        } else {
            return Result::Stuck;
        }
    }

    fn variables(&self) -> &VariableSet {
        &self.variables
    }

    fn description(&self) -> &String {
        return &self.description;
    }
}
