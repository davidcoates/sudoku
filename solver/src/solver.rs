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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::constraints::*;
    use std::rc::Rc;

    struct ReporterImpl {}

    impl Reporter for ReporterImpl {

        fn variable_name(&self, _id: Variable) -> &String {
            panic!("unimplemented");
        }

        fn constraint_name(&self, _id: ConstraintID) -> &String {
            panic!("unimplemented");
        }

        fn emit(&mut self, _breadcrumb: String) {
            panic!("unimplemented");
        }

        fn enabled(&self) -> bool {
            false
        }
    }

    fn encode(r: usize, c: usize) -> usize {
        r * 9 + c
    }

/*
    fn decode(rc: usize) -> (usize, usize) {
        (rc / 9, rc % 9)
    }
*/

    fn sudoku_domains_from_grid(grid: [[usize; 9]; 9]) -> Domains {
        let mut domains = Domains::new();
        for r in 0..9 {
            for c in 0..9 {
                if grid[r][c] == 0 {
                    domains.push(Domain::range(1, 9));
                } else {
                    domains.push(Domain::single(grid[r][c]));
                }
            }
        }
        return domains;
    }

    fn sudoku_constraints() -> Constraints {
        let mut constraints = Constraints::new();
        let domain = Domain::range(1, 9);
        for r in 0..9 {
            let mut variables = VariableSet::new();
            for c in 0..9 {
                variables.insert(encode(r, c));
            }
            constraints.push(BoxedConstraint::new(Rc::new(Permutation::new(0, variables, domain))));
        }
        for r in 0..9 {
            let mut variables = VariableSet::new();
            for c in 0..9 {
                variables.insert(encode(c, r));
            }
            constraints.push(BoxedConstraint::new(Rc::new(Permutation::new(0, variables, domain))));
        }
        for r in 0..3 {
            for c in 0..3 {
                let mut variables = VariableSet::new();
                for i in 0..3 {
                    for j in 0..3 {
                        variables.insert(encode(r*3 + i, c*3 + j));
                    }
                }
                constraints.push(BoxedConstraint::new(Rc::new(Permutation::new(0, variables, domain))));
            }
        }
        return constraints;
    }

    fn check_grid(domains: Domains, expected: [[usize; 9]; 9]) {
        for r in 0..9 {
            for c in 0..9 {
                assert_eq!(*domains.get(encode(r, c)).unwrap(), Domain::single(expected[r][c]));
            }
        }
    }

    #[test]
    fn test_simple_sudoku() {

        let mut reporter = ReporterImpl{};
        let config = Config{
            strict: true,
            max_depth: 1,
        };

        let domains = sudoku_domains_from_grid([
            [0, 0, 0, 1, 0, 2, 0, 0, 0],
            [0, 6, 0, 0, 0, 0, 0, 7, 0],
            [0, 0, 8, 0, 0, 0, 9, 0, 0],
            [4, 0, 0, 0, 0, 0, 0, 0, 3],
            [0, 5, 0, 0, 0, 7, 0, 0, 0],
            [2, 0, 0, 0, 8, 0, 0, 0, 1],
            [0, 0, 9, 0, 0, 0, 8, 0, 5],
            [0, 7, 0, 0, 0, 0, 0, 6, 0],
            [0, 0, 0, 3, 0, 4, 0, 0, 0],
        ]);
        let constraints = sudoku_constraints();

        let mut puzzle = Puzzle::new(domains, constraints);
        let result = puzzle.solve(&mut reporter, config);
        assert!(matches!(result, Result::Solved));

        let expected = [
            [9, 3, 4, 1, 7, 2, 6, 5, 8],
            [5, 6, 1, 9, 4, 8, 3, 7, 2],
            [7, 2, 8, 6, 3, 5, 9, 1, 4],
            [4, 1, 7, 2, 6, 9, 5, 8, 3],
            [8, 5, 3, 4, 1, 7, 2, 9, 6],
            [2, 9, 6, 5, 8, 3, 7, 4, 1],
            [1, 4, 9, 7, 2, 6, 8, 3, 5],
            [3, 7, 2, 8, 5, 1, 4, 6, 9],
            [6, 8, 5, 3, 9, 4, 1, 2, 7],
        ];

        check_grid(puzzle.domains, expected);
    }

}
