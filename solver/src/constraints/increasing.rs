use crate::constraint::*;
use crate::types::*;
use std::rc::Rc;

// Strictly increasing digits
#[derive(Clone,Debug)]
pub struct Increasing {
    id: ConstraintID,
    variables: Vec<Variable>,
    variable_set: VariableSet,
}

impl Increasing {

    pub fn new(id: ConstraintID, variables: Vec<Variable>) -> Self {
        if variables.len() <= 1 {
            panic!("bad Increasing")
        }
        let mut variable_set = VariableSet::new();
        for variable in variables.iter() {
            variable_set.insert(*variable);
        }
        return Increasing {
            id,
            variables,
            variable_set,
        };
    }

}

impl Constraint for Increasing {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let mut last : Option<usize> = None;
        for variable in self.variables.iter() {
            let value = domains.get(*variable).unwrap().value_unchecked();
            if last.is_some() && value <= last.unwrap() {
                return false;
            }
            last = Some(value);
        }
        return true;
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {

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
                        if reporter.enabled() {
                            reporter.emit(format!("{} is not {} considering increasing min of {}", reporter.variable_name(*variable), old.difference(*new), reporter.constraint_name(self.id)));
                        }
                    }
                }
                _ => {}
            }

            let domain = *domains.get(*variable).unwrap();
            if domain.empty() {
                return Result::Unsolvable;
            }
            min = Some(domain.min());
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
                        if reporter.enabled() {
                            reporter.emit(format!("{} is not {} considering decreasing max of {}", reporter.variable_name(*variable), old.difference(*new), reporter.constraint_name(self.id)));
                        }
                    }
                }
                _ => {}
            }

            let domain = *domains.get(*variable).unwrap();
            if domain.empty() {
                return Result::Unsolvable;
            }
            max = Some(domain.max());
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

    fn id(&self) -> ConstraintID {
        self.id
    }
}

