use crate::types::*;

use std::boxed::Box;


pub type Constraints = Vec<Box<dyn Constraint>>;

#[derive(Debug)]
pub enum SimplifyResult {
    Unsolvable,
    Solved,
    Stuck,
    Progress,
    Rewrite(Constraints),
}

pub trait Constraint : std::fmt::Debug {

    fn clone_box(&self) -> Box<dyn Constraint>;

    fn id(&self) -> ConstraintID;

    fn variables(&self) -> &VariableSet;

    // provided not all variables are solved, and no variable is unsolvable,
    // try to simplify by reducing the domain of variables and/or replace the constraint with new (smaller) constraint(s).
    fn simplify(&self, domains: &mut Domains, reporter: &dyn Reporter) -> SimplifyResult;

    // provided all variables are solved, is this constraint satisfied?
    fn check_solved(&self, domains: &mut Domains) -> bool;

    fn check(&self, domains: &mut Domains, reporter: &dyn Reporter)-> Option<SimplifyResult> {
        let mut all_solved = true;
        for variable in self.variables().iter() {
            if domains[variable].len() == 0 {
                if reporter.enabled() {
                    reporter.emit(format!("{} is empty", reporter.variable_name(variable)));
                }
                return Some(SimplifyResult::Unsolvable);
            } else if domains[variable].len() != 1 {
                all_solved = false;
            }
        }
        if all_solved {
            if self.check_solved(domains) {
                return Some(SimplifyResult::Solved);
            } else {
                if reporter.enabled() {
                    reporter.emit(format!("{} is unsolved", reporter.constraint_name(self.id())));
                }
                return Some(SimplifyResult::Unsolvable);
            }
        }
        return None;
    }

    fn check_and_simplify(&self, domains: &mut Domains, reporter: &dyn Reporter) -> SimplifyResult {
        match self.check(domains, reporter) {
            Some(result) => result,
            None => self.simplify(domains, reporter),
        }
    }
}

impl Clone for Box<dyn Constraint> {
    fn clone(&self) -> Self { self.clone_box() }
}

pub fn apply<C, F>(constraint: &C, domains: &mut Domains, reporter: &dyn Reporter, variable: Variable, fun: F) -> bool
where
    C: Constraint,
    F: Fn(&mut Domain)
{
    let new = &mut domains[variable];
    let old = *new;
    fun(new);
    if *new == old {
        return false;
    } else {
        if reporter.enabled() {
            reporter.emit(format!("{} is not {} by {}", reporter.variable_name(variable), old.difference(*new), reporter.constraint_name(constraint.id())));
        }
        return true;
    }

}
