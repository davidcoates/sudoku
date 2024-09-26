use crate::constraint::*;
use crate::types::*;
use crate::constraints::permutation::*;
use crate::bit_set::*;

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

    fn clone_box(&self) -> Box<dyn Constraint> { Box::new(self.clone()) }

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let union : Domain = self.variables.iter().map(|v| domains[v]).union();
        return union.len() == self.variables.len() && (union.max() - union.min() + 1) == union.len();
    }

    fn simplify(&self, domains: &mut Domains, reporter: &dyn Reporter) -> SimplifyResult {

        match simplify_distinct(domains, self.variables) {
            Some((v1, d1)) => {

                let mut progress = false;

                for variable in v1.iter() {
                    progress |= apply(&*self, domains, reporter, variable, |d| d.intersect_with(d1));
                }

                let v2 = self.variables.difference(v1);
                for variable in v2.iter() {
                    progress |= apply(&*self, domains, reporter, variable, |d| d.difference_with(d1));
                }

                if progress {
                    return SimplifyResult::Progress;
                }
            }
            None => {}
        }

        let min = self.variables.iter().filter_map(|v| domains[v].value()).min();
        let max = self.variables.iter().filter_map(|v| domains[v].value()).max();

        if min.is_none() || max.is_none() {
            return SimplifyResult::Stuck;
        }

        // The set must contain min..=max
        // And we can say how much it can be extended either side
        let included_run = max.unwrap() - min.unwrap() + 1;

        if included_run > self.variables.len() {
            if reporter.enabled() {
                reporter.emit(format!("impossible {}", reporter.constraint_name(self.id)));
            }
            return SimplifyResult::Unsolvable;
        }

        let excess = self.variables.len() - included_run;

        let cover = Domain::range(min.unwrap().saturating_sub(excess), max.unwrap().saturating_add(excess));

        let mut progress = false;

        for variable in self.variables.iter() {
            progress |= apply(&*self, domains, reporter, variable, |d| d.intersect_with(cover));
        }

        if progress {
            return SimplifyResult::Progress;
        } else {
            return SimplifyResult::Stuck;
        }
    }

    fn variables(&self) -> &VariableSet {
        &self.variables
    }

    fn id(&self) -> ConstraintID {
        self.id
    }
}
