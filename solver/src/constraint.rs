use std::rc::Rc;

use crate::types::*;
use crate::bit_set::*;

pub trait Tracker {

    fn variable_description(&self) -> String;

    fn on_progress(&mut self, comment: String);
}

pub enum Result {
    Unsolvable,
    Solved,
    Stuck,
    Progress(Vec<BoxedConstraint>)
}

pub type ConstraintID = usize;

pub trait Constraint {

    fn variables(&self) -> &Variables;

    fn simplify(&self, domains: &mut Domains, tracker: &mut Tracker) -> Result;

    fn description(&self) -> String;
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

    pub fn simplify(&self, domains: &mut Domains) -> Result {
        match self.check(domains) {
            Some(result) => result,
            None => self.unbox().simplify(domains),
        }
    }

}

fn progress_simplify(constraint: BoxedConstraint, domains: &mut Domains) -> Result {
    match constraint.simplify(domains) {
        Result::Stuck => {
            return Result::Progress(vec![constraint]);
        },
        r => r
    }
}

fn join(c1: BoxedConstraint, c2: BoxedConstraint, domains: &mut Domains) -> Result {
    let mut r1 = progress_simplify(c1, domains);
    let mut r2 = progress_simplify(c2, domains);
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
#[derive(Clone,Copy)]
pub struct Permutation {
    description: String,
    variables: Variables,
    domain: Domain,
}

impl Permutation {

    pub fn new(description: String, variables: Variables, domain: Domain) -> Self {
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

fn intersect_with(variable: Variable, domains: &mut Domains, domain: Domain) -> bool {
    let new = domains.get_mut(variable).unwrap();
    let old = *new;
    new.intersect_with(domain);
    return *new != old;
}

fn difference_with(variable: Variable, domains: &mut Domains, domain: Domain) -> bool {
    let new = domains.get_mut(variable).unwrap();
    let old = *new;
    new.difference_with(domain);
    return *new != old;
}

impl Constraint for Permutation {

    fn simplify(&self, domains: &mut Domains) -> Result {

        let mut progress = false;
        let mut breadcrumbs = Breadcrumbs::new();

        // Firstly, intersect against this constraint's domain
        for variable in self.variables.iter() {
            let new = domains.get_mut(variable).unwrap();
            let old = *new;
            new.intersect_with(self.domain);
            if *new != old {
                progress = true;
                breadcrumbs.push(Breadcrumb{
                    constraint_id: self.id(),
                    variable: variable,
                    domain_from: old,
                    domain_to: *new,
                });
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
                let selection_complement = self.variables.difference(selection);
                let c1 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, selection, union)));
                let c2 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, selection_complement, self.domain.difference(union))));
                return join(c1, c2, domains);
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
                    let c1 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, selection, intersection)));
                    let c2 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, selection_complement, self.domain.difference(intersection))));
                    return join(c1, c2, domains);
                }
            }
        }

        if progress {
            return Result::Progress(vec![BoxedConstraint::new(Rc::new(*self))]);
        } else {
            return Result::Stuck;
        }
    }

    fn variables(&self) -> &Variables {
        &self.variables
    }

    fn id(&self) -> ConstraintID {
        return self.id;
    }
}


// Strictly increasing digits
#[derive(Clone,Copy)]
pub struct Increasing {
    description: String,
    variables: Variables,
}

impl Increasing {

    pub fn new(description: String, variables: Variables) -> Self {
        return Increasing {
            description,
            variables,
        };
    }

}

impl Constraint for Increasing {

    fn simplify(&self, domains: &mut Domains) -> Result {

        let mut progress = false;
        let mut breadcrumbs = Breadcrumbs::new();

        // Restrict small values
        let mut min : Option<usize> = None;
        for variable in self.variables.iter() {
            match min {
                Some(n) => {
                    let new = domains.get_mut(variable).unwrap();
                    let old = *new;
                    new.difference_with(Domain::range(0, n));
                    if *new != old {
                        progress = true;
                        breadcrumbs.push(Breadcrumb{
                            constraint_id: self.id(),
                            variable: variable,
                            domain_from: old,
                            domain_to: *new,
                        });
                    }
                }
                _ => {}
            }

            min = Some(domains.get(variable).unwrap().min());
        }
/*
        // Restrict large values
        let mut max : Option<usize> = None;
        for variable in self.variables.iter().rev() {
            match max {
                Some(n) => {
                    let new = domains.get_mut(variable).unwrap();
                    let old = *new;
                    new.intersect_with(Domain::range(0, n - 1));
                    if *new != old {
                        progress = true;
                        breadcrumbs.push(Breadcrumb{
                            constraint_id: self.id(),
                            variable: variable,
                            domain_from: old,
                            domain_to: *new,
                        });
                    }
                }
                _ => {}
            }

            max = Some(domains.get(variable).unwrap().max());
        }
*/
        if progress {
            return Result::Progress(vec![BoxedConstraint::new(Rc::new(*self))]);
        } else {
            return Result::Stuck;
        }
    }

    fn variables(&self) -> &Variables {
        &self.variables
    }

    fn id(&self) -> ConstraintID {
        return self.id;
    }
}
