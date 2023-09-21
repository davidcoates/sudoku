use crate::types::*;
use crate::constraint::*;

pub struct Puzzle {
    pub domains: Domains,
    pub constraints: Constraints,
    pub depth: u64,
}

// TODO could optimise this for:
// only solved constraints
// order of constraints
// only constraints with dirty variables
fn simplify(domains: &mut Domains, constraints: &Constraints, reporter: &mut dyn Reporter) -> Result {
    // TODO
    let mut new_constraints = Constraints::new();
    let mut any_progress = false;
    for constraint in constraints {
        let result = constraint.simplify(domains, reporter);
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

    pub fn new(domains: Domains, constraints: Constraints) -> Self {
        Puzzle {
            domains: domains,
            constraints: constraints,
            depth: 1,
        }
    }

    pub fn solve_no_branch(self: &mut Puzzle, reporter: &mut dyn Reporter, config: Config) -> Result {
        let result = simplify(&mut self.domains, &self.constraints, reporter);
        match result {
            Result::Progress(constraints) => { self.constraints = constraints; return self.solve_no_branch(reporter, config); },
            _ => result,
        }
    }

    pub fn solve(self: &mut Puzzle, reporter: &mut dyn Reporter, config: Config) -> Result {
        let result = self.solve_no_branch(reporter, config);
        if self.depth > config.max_depth {
            return result;
        }
        match result {
            Result::Stuck => {

                // Heuristic = variable with smallest domain
                // TODO use something smarter here, e.g. most constrained variables
                let mut variables = self.domains.iter().enumerate()
                    .filter(|(_, domain)| domain.len() > 1)
                    .collect::<Vec<_>>();
                variables
                    .sort_by(|(_, d1), (_, d2)| d1.len().cmp(&d2.len()));
                let mut variables = variables.iter()
                    .map(|(v, _)| *v)
                    .collect::<Vec<_>>();

                // Sort again, looking at most constrained
                let num_constraints = |v| self.constraints.iter().filter(|c| c.unbox().variables().contains(v)).count();
                variables.sort_by(|v1, v2| num_constraints(*v2).cmp(&num_constraints(*v1)));

                for variable in variables.iter() {
                    let domain = *self.domains.get(*variable).unwrap();
                    let mut new_domain : Domain = domain;
                    for value in domain.iter() {
                        // Guess variable = value and try to solve without branching
                        let mut puzzle = Puzzle {
                            domains: self.domains.clone(),
                            constraints: self.constraints.clone(), // TODO do constraints really need to be cloned?
                            depth: self.depth + 1,
                        };
                        *puzzle.domains.get_mut(*variable).unwrap() = Domain::single(value);
                        if reporter.enabled() {
                            reporter.emit(format!("guess {} = {}", reporter.variable_name(*variable), value));
                        }
                        let result = puzzle.solve_no_branch(reporter, config);
                        match result {
                            Result::Unsolvable => { new_domain.remove(value); },
                            Result::Solved => if config.strict {} else { *self = puzzle; return Result::Solved; },
                            _ => {},
                        }
                    }
                    if new_domain != domain {
                        if reporter.enabled() {
                            reporter.emit(format!("{} is {} by guessing", reporter.variable_name(*variable), new_domain));
                        }
                        *self.domains.get_mut(*variable).unwrap() = new_domain;
                        return self.solve(reporter, config);
                    }
                }

                return Result::Stuck;
            },
            _ => result,
        }
    }

}
