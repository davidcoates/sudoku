use crate::types::*;
use crate::constraint::*;

pub struct Puzzle {
    pub domains: Domains,
    pub constraints: Constraints,
}

// TODO could optimise this for:
// only solved constraints
// order of constraints
// only constraints with dirty variables
fn simplify(domains: &mut Domains, constraints: &Constraints, tracker: &mut dyn Tracker) -> Result {
    // TODO
    let mut new_constraints = Constraints::new();
    let mut any_progress = false;
    for constraint in constraints {
        let result = constraint.simplify(domains, tracker);
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

    pub fn solve_no_branch(self: &mut Puzzle, tracker: &mut dyn Tracker) -> Result {
        let result = simplify(&mut self.domains, &self.constraints, tracker);
        match result {
            Result::Progress(constraints) => { self.constraints = constraints; return self.solve(tracker); },
            _ => result,
        }
    }

    pub fn solve(self: &mut Puzzle, tracker: &mut dyn Tracker) -> Result {
        let result = self.solve_no_branch(tracker);
        match result {
            Result::Stuck => {

                // Heuristic = variable with smallest domain
                // TODO use something smarter here, e.g. most constrained variables
                let mut variables = self.domains.iter().enumerate()
                    .filter(|(_, domain)| domain.len() > 1)
                    .collect::<Vec<_>>();
                variables
                    .sort_by(|(_, d1), (_, d2)| d1.len().cmp(&d2.len()));
                let variables = variables.iter()
                    .map(|(v, _)| *v)
                    .collect::<Vec<_>>();

                for variable in variables.iter() {
                    let domain = *self.domains.get(*variable).unwrap();
                    let mut new_domain : Domain = domain;
                    for value in domain.iter() {
                        // Guess variable = value and try to solve without branching
                        let mut puzzle = Puzzle {
                            domains: self.domains.clone(),
                            constraints: self.constraints.clone(),
                        };
                        *puzzle.domains.get_mut(*variable).unwrap() = Domain::single(value);
                        let result = puzzle.solve_no_branch(tracker);
                        match result {
                            Result::Unsolvable => { new_domain.remove(value); },
                            // Note: Assumes there is a unique solution!
                            Result::Solved => { *self = puzzle; return Result::Solved; }
                            _ => {},
                        }
                    }
                    if new_domain != domain {
                        *self.domains.get_mut(*variable).unwrap() = new_domain;
                        return self.solve(tracker);
                    }
                }

                return Result::Stuck;
            },
            _ => result,
        }
    }

}
