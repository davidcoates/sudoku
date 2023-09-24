use crate::constraint::*;
use crate::types::*;
use crate::constraints::permutation::*;
use std::rc::Rc;

#[derive(Clone,Debug)]
pub struct ConsecutiveSet {
    id: ConstraintID,
    variables: VariableSet,
}

impl ConsecutiveSet {

    pub fn new(id: ConstraintID, variables: VariableSet) -> Self {
        if variables.len() <= 1 {
            panic!("bad ConsecutiveSet")
        }
        return ConsecutiveSet {
            id,
            variables,
        };
    }

}

impl Constraint for ConsecutiveSet {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let mut domain = Domain::new();
        for variable in self.variables.iter() {
            domain.union_with(*domains.get(variable).unwrap());
        }
        return domain.len() == self.variables.len() && (domain.max() - domain.min() + 1) == domain.len();
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {

        match simplify_distinct(domains, self.variables) {
            Some((v1, d1)) => {

                let mut progress = false;

                for variable in v1.iter() {
                    let new = domains.get_mut(variable).unwrap();
                    let old = *new;
                    new.intersect_with(d1);
                    if *new != old {
                        progress = true;
                        if reporter.enabled() {
                            reporter.emit(format!("{} is not {} by {}", reporter.variable_name(variable), old.difference(*new), reporter.constraint_name(self.id)));
                        }
                    }
                }

                let v2 = self.variables.difference(v1);
                for variable in v2.iter() {
                    let new = domains.get_mut(variable).unwrap();
                    let old = *new;
                    new.difference_with(d1);
                    if *new != old {
                        progress = true;
                        if reporter.enabled() {
                            reporter.emit(format!("{} is not {} by {}", reporter.variable_name(variable), old.difference(*new), reporter.constraint_name(self.id)));
                        }
                    }
                }

                if progress {
                    return Result::Progress(vec![BoxedConstraint::new(self)]);
                }
            }
            None => {}
        }

        let min = self.variables.iter().filter_map(|v| domains.get(v).unwrap().value()).min();
        let max = self.variables.iter().filter_map(|v| domains.get(v).unwrap().value()).max();

        if min.is_none() || max.is_none() {
            return Result::Stuck;
        }

        // The set must contain min..=max
        // And we can say how much it can be extended either side
        let included_run = max.unwrap() - min.unwrap() + 1;

        if included_run > self.variables.len() {
            if reporter.enabled() {
                reporter.emit(format!("impossible {}", reporter.constraint_name(self.id))); // self.variables, self.variables.iter().map(|v| *domains.get(v).unwrap()).collect::<Vec<_>>() ));
            }
            return Result::Unsolvable;
        }

        let excess = self.variables.len() - included_run;

        let cover = Domain::range(min.unwrap().saturating_sub(excess), max.unwrap().saturating_add(excess));

        let mut progress = false;

        for variable in self.variables.iter() {
            let new = domains.get_mut(variable).unwrap();
            let old = *new;
            new.intersect_with(cover);
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
