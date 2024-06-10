use crate::constraint::*;
use crate::types::*;
use crate::bit_set::*;
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
            let value = domains[variable].value_unchecked();
            if last.is_some() && value != last.unwrap() {
                return false;
            }
            last = Some(value);
        }
        return true;
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &Reporter) -> Result {

        // Compute the intersection of all domains
        let intersection = self.variables.iter().map(|v| domains[v]).intersection();

        let mut progress = false;
        for variable in self.variables.iter() {
            progress |= apply(&*self, domains, reporter, variable, |d| d.intersect_with(intersection));
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

