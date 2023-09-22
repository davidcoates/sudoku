use crate::constraint::*;
use crate::types::*;
use std::rc::Rc;

#[derive(Clone,Debug)]
pub struct Equals {
    id: ConstraintID,
    variables: VariableSet,
}

impl Equals {

    pub fn new(id: ConstraintID, variables: VariableSet) -> Self {
        if variables.len() <= 1 {
            panic!("bad Equals")
        }
        return Equals {
            id,
            variables,
        };
    }

}

impl Constraint for Equals {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let mut last : Option<usize> = None;
        for variable in self.variables.iter() {
            let value = domains.get(variable).unwrap().value_unchecked();
            if last.is_some() && value != last.unwrap() {
                return false;
            }
            last = Some(value);
        }
        return true;
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {

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
                if reporter.enabled() {
                    reporter.emit(format!("{} is not {} by {}", reporter.variable_name(variable), old.difference(*new), reporter.constraint_name(self.id)));
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

    fn id(&self) -> ConstraintID {
        self.id
    }
}

