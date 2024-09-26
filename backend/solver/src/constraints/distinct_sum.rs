use crate::constraint::*;
use crate::constraints::permutation::*;
use crate::types::*;
use crate::bit_set::*;

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

    fn clone_box(&self) -> Box<dyn Constraint> { Box::new(self.clone()) }

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let union : Domain = self.variables.iter().map(|v| domains[v]).union();
        return union.len() == self.variables.len() && self.sum == union.iter().sum::<usize>();
    }

    fn simplify(&self, domains: &mut Domains, reporter: &dyn Reporter) -> SimplifyResult {

        if self.variables.len() == 1 {
            let variable = self.variables.iter().next().unwrap();
            let domain = &mut domains[variable];
            if reporter.enabled() {
                reporter.emit(format!("{} is {} by {}", reporter.variable_name(variable), self.sum, reporter.constraint_name(self.id)));
            }
            domain.intersect_with(Domain::single(self.sum));
            if domain.empty() {
                return SimplifyResult::Unsolvable;
            } else {
                return SimplifyResult::Solved;
            }
        }

        match simplify_distinct(domains, self.variables) {
            Some((v1, d1)) => {
                let sum : usize = d1.iter().sum();
                if sum > self.sum {
                    return SimplifyResult::Unsolvable;
                }
                for variable in v1.iter() {
                    domains[variable].intersect_with(d1);
                }
                let v2 = self.variables.difference(v1);
                for variable in v2.iter() {
                    domains[variable].difference_with(d1);
                }
                let c1 = Box::new(Permutation::new(self.id, v1, d1));
                let c2 = Box::new(DistinctSum::new(self.id, v2, self.sum - sum));
                return SimplifyResult::Rewrite(vec![c1, c2]);
            }
            _ => {
                // simplify_distinct doesn't consider n tuple (where n = variables.len()),
                // so consider that for ourselves.
                let union: Domain = self.variables.iter().map(|v| domains[v]).union();
                if union.len() == self.variables.len() {
                    if self.sum == union.iter().sum::<usize>() {
                       let constraint = Box::new(Permutation::new(self.id, self.variables, union));
                       return SimplifyResult::Rewrite(vec![constraint]);
                    } else {
                        return SimplifyResult::Unsolvable;
                    }
                } else {
                    return SimplifyResult::Stuck;
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

