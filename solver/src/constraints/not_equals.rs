use crate::constraint::*;
use crate::types::*;
use std::rc::Rc;

#[derive(Clone,Debug)]
pub struct NotEquals {
    id: ConstraintID,
    variables: VariableSet,
}

impl NotEquals {

    pub fn new(id: ConstraintID, variables: VariableSet) -> Self {
        if variables.len() != 2 { // TODO support > 2 (in the simplifier)
            panic!("bad NotEquals")
        }
        return NotEquals {
            id,
            variables,
        };
    }

}

impl Constraint for NotEquals {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let mut seen = Domain::new();
        for variable in self.variables.iter() {
            let domain = *domains.get(variable).unwrap();
            if !seen.intersection(domain).empty() {
                return false;
            }
            seen.union_with(domain);
        }
        return true;
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {

        let mut iter = self.variables.iter();
        let v1 = iter.next().unwrap();
        let v2 = iter.next().unwrap();

        let d1 = *domains.get(v1).unwrap();
        let d2 = *domains.get(v2).unwrap();

        if d1.len() == 1 && d2.len() == 1 {
            if d1.value_unchecked() == d2.value_unchecked() {
                return Result::Unsolvable;
            } else {
                return Result::Solved;
            }
        }

        let mut progress = false;

        if d1.len() == 1 {
            let value = d1.value_unchecked();
            if d2.contains(value) {
                progress = true;
                domains.get_mut(v2).unwrap().remove(value);
                if reporter.enabled() {
                    reporter.emit(format!("{} is not {} by {}", reporter.variable_name(v2), value, reporter.constraint_name(self.id)));
                }
            }
        }

        if d2.len() == 1 {
            let value = d2.value_unchecked();
            if d1.contains(value) {
                progress = true;
                domains.get_mut(v1).unwrap().remove(value);
                if reporter.enabled() {
                    reporter.emit(format!("{} is not {} by {}", reporter.variable_name(v1), value, reporter.constraint_name(self.id)));
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

