use crate::domain::*;
use crate::constraint::*;

pub struct Puzzle {
    pub domains: Domains,
    pub constraints: Constraints,
}

// TODO could optimise this for:
// only solved constraints
// order of constraints
// only constraints with dirty variables
fn simplify(domains: &mut Domains, constraints: &Constraints) -> Result {
    // TODO
    let mut new_constraints = Constraints::new();
    let mut any_progress = false;
    for constraint in constraints {
        let result = constraint.simplify(domains);
        match result {
            Result::Unsolvable                    => { return Result::Unsolvable; },
            Result::Solved                        => {},
            Result::Stuck                         => { new_constraints.push(constraint.clone()); },
            Result::Progress(mut sub_constraints) => { new_constraints.append(&mut sub_constraints); any_progress = true; }
        }
    }
    if new_constraints.is_empty() {
        return Result::Solved;
    } else if any_progress {
        return Result::Progress(new_constraints);
    } else {
        return Result::Stuck;
    }
}

// TODO branch...

impl Puzzle {

    pub fn solve(self: &mut Puzzle) -> Result {
        let result = simplify(&mut self.domains, &self.constraints);
        match result {
            Result::Progress(constraints) => { self.constraints = constraints; return self.solve(); },
            _ => result,
        }
    }

}
