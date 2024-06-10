use crate::bit_set::*;
use crate::constraint::*;
use crate::types::*;
use std::rc::Rc;

// Distinct digits covering a domain
#[derive(Clone,Debug)]
pub struct Permutation {
    id: ConstraintID,
    variables: VariableSet,
    domain: Domain,
}

impl Permutation {

    pub fn new(id: ConstraintID, variables: VariableSet, domain: Domain) -> Self {
        if variables.len() != domain.len() {
            panic!("bad Permutation: #variables != #domain")
        }
        return Permutation {
            id,
            variables,
            domain
        };
    }

}

pub fn simplify_distinct(domains: &mut Domains, variables: VariableSet) -> Option<(VariableSet, Domain)> {

    // Solve based on the following idea:
    //
    // Suppose there is a set S of n cells, and the union of domains of S is I.
    // If:
    //  * I has length n
    // Then:
    //  * The domain of each cell not in S can be subtracted by I.
    //
    // E.g. if there are three cells A = (12) and B = (23), and C = (13), then (123) can be removed from all other cells.

    let variable_list: Vec<usize> = variables.iter().collect();
    for combination in 1..(u128::pow(2, variable_list.len() as u32) - 1) {
        let selection: VariableSet = VariableSet::from_bits(combination).iter().map(|i| VariableSet::single(variable_list[i])).union();
        let     union:     Domain  =      Domain::from_bits(combination).iter().map(|i| domains[variable_list[i]]).union();
        if union.len() == selection.len() {
            return Some((selection, union));
        }
    }

    return None;
}

impl Constraint for Permutation {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let union: Domain = self.variables.iter().map(|v| domains[v]).union();
        return union == self.domain;
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &dyn Reporter) -> Result {

        let mut progress = false;

        // Firstly, intersect against this constraint's domain
        for variable in self.variables.iter() {
            progress |= apply(&*self, domains, reporter, variable, |d| d.intersect_with(self.domain));
        }

        //match simplify_distinct(domains, self.variables).or_else(|| simplify_permutation(domains, self.variables)) {
        match simplify_distinct(domains, self.variables) {
            Some((v1, d1)) => {
                let c1 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, v1, d1)));
                let (v2, d2) = (self.variables.difference(v1), self.domain.difference(d1));
                let c2 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, v2, d2)));
                return join(c1, c2, domains, reporter);
            }
            None => {}
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

