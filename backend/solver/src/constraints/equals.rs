use crate::constraint::*;
use crate::types::*;
use crate::bit_set::*;

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

    fn clone_box(&self) -> Box<dyn Constraint> { Box::new(self.clone()) }

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

    fn simplify(&self, domains: &mut Domains, reporter: &dyn Reporter) -> SimplifyResult {

        // Compute the intersection of all domains
        let intersection = self.variables.iter().map(|v| domains[v]).intersection();

        let mut progress = false;
        for variable in self.variables.iter() {
            progress |= apply(&*self, domains, reporter, variable, |d| d.intersect_with(intersection));
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

