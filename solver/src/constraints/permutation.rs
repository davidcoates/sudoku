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

impl Constraint for Permutation {

    fn check_solved(&self, domains: &mut Domains) -> bool {
        let mut domain = Domain::new();
        for variable in self.variables.iter() {
            domain.union_with(*domains.get(variable).unwrap())
        }
        return domain == self.domain;
    }

    fn simplify(self: Rc<Self>, domains: &mut Domains, reporter: &mut dyn Reporter) -> Result {

        let mut progress = false;

        // Firstly, intersect against this constraint's domain
        for variable in self.variables.iter() {
            let new = domains.get_mut(variable).unwrap();
            let old = *new;
            new.intersect_with(self.domain);
            if *new != old {
                progress = true;
                if reporter.enabled() {
                    reporter.emit(format!("{} is not {} by {}", reporter.variable_name(variable), old.difference(*new), reporter.constraint_name(self.id)));
                }
            }
        }

        // Solve based on the following idea:
        //
        // Suppose there is a set S of n cells, and the union of domains of S is I.
        // If:
        //  * I has length n
        // Then:
        //  * The domain of each cell not in S can be subtracted by I.
        //
        // E.g. if there are three cells A = (12) and B = (23), and C = (13), then (123) can be removed from all other cells.

        let values: Vec<usize> = self.variables.iter().collect();
        for combination in 1..(u128::pow(2, values.len() as u32) - 1) {
            let mut selection = BitSet::new();
            let mut union = Domain::new();
            for i in BitSet::from_bits(combination).iter() {
                let variable = *values.get(i).unwrap();
                selection.insert(variable);
                union.union_with(*domains.get(variable).unwrap());
            }
            if union.len() == selection.len() {
                let c1 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, selection, union)));
                let selection_complement = self.variables.difference(selection);
                let c2 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, selection_complement, self.domain.difference(union))));
                return join(c1, c2, domains, reporter);
            }
        }


        // Solve based on the following idea:
        //
        // Suppose there is a set S of n cells, and the intersection of domains of S is I.
        // If:
        //  * I does not overlap with the domain of any cell not in S, and
        //  * I has length n
        // Then:
        //  * The domain of each cell in S can be intersected with I
        //  * The domain of each cell not in S can be subtracted by I.
        //
        // E.g. if there are two cells A = (123) and B = (124), and no other cell contains a 1 or a 2,
        // then A = B = (12), and both 1 and 2 are removed from the domain of every remaining cell.

        let values: Vec<usize> = self.variables.iter().collect();
        for combination in 1..(u128::pow(2, values.len() as u32) - 1) {
            let mut selection = Domain::new();
            let mut intersection = Domain::all();
            for i in Domain::from_bits(combination).iter() {
                let variable = *values.get(i).unwrap();
                selection.insert(variable);
                intersection.intersect_with(*domains.get(variable).unwrap());
            }
            if intersection.len() == selection.len() {
                let mut ok = true;
                let selection_complement = self.variables.difference(selection);
                for variable in selection_complement.iter() {
                    if !domains.get(variable).unwrap().intersection(intersection).empty() {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    let c1 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, selection, intersection)));
                    let c2 = BoxedConstraint::new(Rc::new(Permutation::new(self.id, selection_complement, self.domain.difference(intersection))));
                    return join(c1, c2, domains, reporter);
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

