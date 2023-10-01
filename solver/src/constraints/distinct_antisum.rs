use crate::constraint::*;
use crate::constraints::permutation::*;
use crate::types::*;
use crate::bit_set::*;
use std::rc::Rc;

#[derive(Clone,Debug)]
pub struct DistinctAntisum {
    id: ConstraintID,
    variables: VariableSet,
    antisums: Domain,
}

impl DistinctAntisum {

    pub fn new(id: ConstraintID, variables: VariableSet, antisums: Domain) -> Self {
        if variables.len() == 0 {
            panic!("bad DistinctAntisum");
        }
        return DistinctAntisum {
            id,
            variables,
            antisums,
        };
    }

}

impl Constraint for DistinctAntisum {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let union : Domain = self.variables.iter().map(|v| domains[v]).union();
        return union.len() == self.variables.len() && !self.antisums.contains(union.iter().sum::<usize>());
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &dyn Reporter) -> Result {

        if self.variables.len() == 1 {
            let variable = self.variables.iter().next().unwrap();
            let domain = &mut domains[variable];
            if reporter.enabled() {
                reporter.emit(format!("{} is not {} by {}", reporter.variable_name(variable), self.antisums, reporter.constraint_name(self.id)));
            }
            domain.difference_with(self.antisums);
            if domain.empty() {
                return Result::Unsolvable;
            } else {
                return Result::Solved;
            }
        }

        // TODO special case for 2 variables?

        match simplify_distinct(domains, self.variables) {
            Some((v1, d1)) => {
                let sum : usize = d1.iter().sum();
                let mut new_antisums = Domain::new();
                for antisum in self.antisums.iter() {
                    if antisum >= sum {
                        new_antisums.insert(antisum - sum);
                    }
                }
                for variable in v1.iter() {
                    domains[variable].intersect_with(d1);
                }
                let v2 = self.variables.difference(v1);
                for variable in v2.iter() {
                    domains[variable].difference_with(d1);
                }
                let c1 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, v1, d1)));
                let c2 = BoxedConstraint::new(Rc::new(DistinctAntisum::new(self.id, v2, new_antisums)));
                return join(c1, c2, domains, reporter);
            }
            _ => {
                // simplify_distinct doesn't consider n tuple (where n = variables.len()),
                // so consider that for ourselves.
                let union: Domain = self.variables.iter().map(|v| domains[v]).union();
                if union.len() == self.variables.len() {
                    if !self.antisums.contains(union.iter().sum::<usize>()) {
                       let constraint = BoxedConstraint::new(Rc::new(Permutation::new(self.id, self.variables, union)));
                       return progress_simplify(constraint, domains, reporter);
                    } else {
                        return Result::Unsolvable;
                    }
                } else {
                    return Result::Stuck;
                }
            }
        }
    }

    fn variables(&self) -> &VariableSet {
        &self.variables
    }

    fn id(&self) -> ConstraintID {
        self.id
    }
}

