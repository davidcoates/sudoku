use crate::constraint::*;
use crate::types::*;
use crate::bit_set::*;

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
        let variable_set = variables.iter().map(|v| Domain::single(*v)).union();
        return Difference {
            id,
            variables,
            variable_set,
            threshold: threshold,
        };
    }

}

fn difference(domain: Domain, threshold: usize) -> Domain {
    domain.iter().map(|v|
        // values at least threshold away from value
        Domain::range(v.saturating_sub(threshold - 1), v.saturating_add(threshold - 1)).complement()
    ).union()
}

impl Constraint for Difference {

    fn clone_box(&self) -> Box<dyn Constraint> { Box::new(self.clone()) }

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let mut last : Option<usize> = None;
        for variable in self.variables.iter() {
            let value = domains[*variable].value_unchecked();
            if last.is_some() && usize::abs_diff(value, last.unwrap()) < self.threshold {
                return false;
            }
            last = Some(value);
        }
        return true;
    }

    fn simplify(&self, domains: &mut Domains, reporter: &dyn Reporter) -> SimplifyResult {

        let mut progress = false;

        // Restrict small values
        let mut last : Option<usize> = None;
        for variable in self.variables.iter() {
            match last {
                Some(v2) => {
                    let v1 = *variable;
                    let d1 = domains[v1];
                    let d2 = domains[v2];

                    {
                        progress |= apply(&*self, domains, reporter, v2, |d| d.intersect_with(difference(d1, self.threshold)));
                        if domains[v2].empty() {
                            return SimplifyResult::Stuck;
                        }
                    }

                    {
                        progress |= apply(&*self, domains, reporter, v1, |d| d.intersect_with(difference(d2, self.threshold)));
                        if domains[v1].empty() {
                            return SimplifyResult::Stuck;
                        }
                    }
                },
                _ => {}
            }
            last = Some(*variable);
        }

        if progress {
            return SimplifyResult::Progress;
        } else {
            return SimplifyResult::Stuck;
        }
    }

    fn variables(&self) -> &VariableSet {
        &self.variable_set
    }

    fn id(&self) -> ConstraintID {
        self.id
    }
}

