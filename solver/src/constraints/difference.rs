use crate::constraint::*;
use crate::types::*;
use std::rc::Rc;

// Strictly increasing digits
#[derive(Clone,Debug)]
pub struct Difference {
    id: ConstraintID,
    variables: Vec<Variable>,
    variable_set: VariableSet,
    threshold: usize,
}

impl Difference {

    pub fn new(id: ConstraintID, variables: Vec<Variable>, threshold: usize) -> Self {
        if variables.len() <= 1 {
            panic!("bad Difference")
        }
        let mut variable_set = VariableSet::new();
        for variable in variables.iter() {
            variable_set.insert(*variable);
        }
        return Difference {
            id,
            variables,
            variable_set,
            threshold: threshold,
        };
    }

}

// domain of values at least threshold away from value
fn difference_single(value: usize, threshold: usize) -> Domain {
    return Domain::range(value.saturating_sub(threshold - 1), value.saturating_add(threshold - 1)).complement();
}

fn difference(domain: Domain, threshold: usize) -> Domain {
    let mut union = Domain::new();
    for value in domain.iter() {
        union.union_with(difference_single(value, threshold));
    }
    return union;
}

impl Constraint for Difference {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let mut last : Option<usize> = None;
        for variable in self.variables.iter() {
            let value = domains.get(*variable).unwrap().value_unchecked();
            if last.is_some() && usize::abs_diff(value, last.unwrap()) < self.threshold {
                return false;
            }
            last = Some(value);
        }
        return true;
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {

        let mut progress = false;

        // Restrict small values
        let mut last : Option<usize> = None;
        for variable in self.variables.iter() {
            match last {
                Some(v2) => {
                    let v1 = *variable;
                    let d1 = *domains.get(v1).unwrap();
                    let d2 = *domains.get(v2).unwrap();

                    {
                        let new = domains.get_mut(v2).unwrap();
                        let old = *new;
                        new.intersect_with(difference(d1, self.threshold));
                        if *new != old {
                            progress = true;
                            if reporter.enabled() {
                                reporter.emit(format!("{} is not {} since {}", reporter.variable_name(v2), old.difference(*new), reporter.constraint_name(self.id)));
                            }
                        }
                        if new.empty() {
                            return Result::Stuck;
                        }
                    }

                    {
                        let new = domains.get_mut(v1).unwrap();
                        let old = *new;
                        new.intersect_with(difference(d2, self.threshold));
                        if *new != old {
                            progress = true;
                            if reporter.enabled() {
                                reporter.emit(format!("{} is not {} since {}", reporter.variable_name(v1), old.difference(*new), reporter.constraint_name(self.id)));
                            }
                        }
                        if new.empty() {
                            return Result::Stuck;
                        }
                    }
                },
                _ => {}
            }
            last = Some(*variable);
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

