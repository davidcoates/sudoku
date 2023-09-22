use crate::constraint::*;
use crate::types::*;
use crate::constraints::permutation::Permutation;
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

        // Compute the union of all domains
        let mut cover = Domain::new();
        for variable in self.variables.iter() {
            cover.union_with(*domains.get_mut(variable).unwrap());
        }

        if cover.len() < self.variables.len() {
            return Result::Stuck;
        }

        // Take the union of (sufficient length) runs in the cover
        let mut run_cover = Domain::new();
        for i in cover.min()..=(cover.max() - self.variables.len()) {
            let run = Domain::range(i, i + self.variables.len());
            if run.intersection(cover) == run {
                run_cover.union_with(run);
            }
        }

        if run_cover.len() < self.variables.len() {
            return Result::Stuck;
        }

        if run_cover.len() == self.variables.len() {
            // This just becomes a permutation constraint
            let constraint = BoxedConstraint::new(Rc::new(Permutation::new(self.id, self.variables, run_cover)));
            return Result::Progress(vec![constraint]);
        }

        let mut progress = false;
        if run_cover != cover {
            for variable in self.variables.iter() {
                let new = domains.get_mut(variable).unwrap();
                let old = *new;
                new.intersect_with(run_cover);
                if *new != old {
                    progress = true;
                    if reporter.enabled() {
                        reporter.emit(format!("{} is not {} by {}", reporter.variable_name(variable), old.difference(*new), reporter.constraint_name(self.id)));
                    }
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
