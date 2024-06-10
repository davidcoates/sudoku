use crate::constraint::*;
use crate::constraints::permutation::*;
use crate::types::*;
use crate::bit_set::*;
use std::rc::Rc;

#[derive(Clone,Debug)]
pub struct DistinctSum {
    id: ConstraintID,
    variables: VariableSet,
    sum: usize,
}

impl DistinctSum {

    pub fn new(id: ConstraintID, variables: VariableSet, sum: usize) -> Self {
        if variables.len() == 0 {
            panic!("bad DistinctSum");
        }
        return DistinctSum {
            id,
            variables,
            sum,
        };
    }

}

impl Constraint for DistinctSum {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let union : Domain = self.variables.iter().map(|v| domains[v]).union();
        return union.len() == self.variables.len() && self.sum == union.iter().sum::<usize>();
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &Reporter) -> Result {

        if self.variables.len() == 1 {
            let variable = self.variables.iter().next().unwrap();
            let domain = &mut domains[variable];
            if reporter.enabled() {
                reporter.emit(format!("{} is {} by {}", reporter.variable_name(variable), self.sum, reporter.constraint_name(self.id)));
            }
            domain.intersect_with(Domain::single(self.sum));
            if domain.empty() {
                return Result::Unsolvable;
            } else {
                return Result::Solved;
            }
        }

        match simplify_distinct(domains, self.variables) {
            Some((v1, d1)) => {
                let sum : usize = d1.iter().sum();
                if sum > self.sum {
                    return Result::Unsolvable;
                }
                for variable in v1.iter() {
                    domains[variable].intersect_with(d1);
                }
                let v2 = self.variables.difference(v1);
                for variable in v2.iter() {
                    domains[variable].difference_with(d1);
                }
                let c1 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, v1, d1)));
                let c2 = BoxedConstraint::new(Rc::new(DistinctSum::new(self.id, v2, self.sum - sum)));
                return join(c1, c2, domains, reporter);
            }
            _ => {
                // simplify_distinct doesn't consider n tuple (where n = variables.len()),
                // so consider that for ourselves.
                let union: Domain = self.variables.iter().map(|v| domains[v]).union();
                if union.len() == self.variables.len() {
                    if self.sum == union.iter().sum::<usize>() {
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

