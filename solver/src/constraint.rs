use std::rc::Rc;

use crate::types::*;

#[derive(Debug)]
pub enum Result {
    Unsolvable,
    Solved,
    Stuck,
    Progress(Vec<BoxedConstraint>)
}

pub trait Constraint : std::fmt::Debug {
    fn variables(&self) -> &VariableSet;
    fn id(&self) -> ConstraintID;

    // provided not all variables are solved, and no variable is unsolvable,
    // try to simplify by reducing the domain of variables and/or replace the constraint with new (smaller) constraint(s).
    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result;

    // provided all variables are solved, is this constraint satisfied?
    fn check_solved(&self, domains: &mut Domains) -> bool;
}

pub type Constraints = Vec<BoxedConstraint>;

#[derive(Clone,Debug)]
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

    fn check(&self, domains: &mut Domains, reporter: &mut dyn Reporter)-> Option<Result> {
        let mut all_solved = true;
        for variable in self.unbox().variables().iter() {
            if domains.get(variable).unwrap().len() == 0 {
                if reporter.enabled() {
                    reporter.emit(format!("{} is empty", reporter.variable_name(variable)));
                }
                return Some(Result::Unsolvable);
            } else if domains.get(variable).unwrap().len() != 1 {
                all_solved = false;
            }
        }
        if all_solved {
            if self.unbox().check_solved(domains) {
                return Some(Result::Solved);
            } else {
                if reporter.enabled() {
                    reporter.emit(format!("{} is unsolved", reporter.constraint_name(self.unbox().id())));
                }
                return Some(Result::Unsolvable);
            }
        }
        return None;
    }

    pub fn simplify(&self, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {
        match self.check(domains, reporter) {
            Some(result) => result,
            None => self.constraint.clone().simplify(domains, reporter),
        }
    }

}

fn progress_simplify(constraint: BoxedConstraint, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {
    match constraint.simplify(domains, reporter) {
        Result::Stuck => {
            return Result::Progress(vec![constraint]);
        },
        r => r
    }
}

pub fn join(c1: BoxedConstraint, c2: BoxedConstraint, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {
    let mut r1 = progress_simplify(c1, domains, reporter);
    let mut r2 = progress_simplify(c2, domains, reporter);
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
