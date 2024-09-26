use crate::types::*;
use crate::constraint::*;

use serde::Serialize;
use serde::Deserialize;


#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SolveResult {
    Unsolvable,
    Solved,
    Stuck,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Config {
    pub greedy: bool,
    pub breadcrumbs: bool,
}

pub struct Solver {
    pub variable_names: Vec<String>,
    pub constraint_names: Vec<String>,
    pub config: Config,
}

impl Reporter for Solver {

    fn variable_name(&self, id: Variable) -> &String {
        &self.variable_names[id]
    }

    fn constraint_name(&self, id: ConstraintID) -> &String {
        &self.constraint_names[id]
    }

    fn emit(&self, breadcrumb: String) {
        eprint!("{}\n", breadcrumb);
    }

    fn enabled(&self) -> bool {
        self.config.breadcrumbs
    }

}

// TODO branch...

impl Solver {

    pub fn solve(&self, domains: &mut Domains, constraints: &mut Constraints) -> SolveResult {
        let result = self.simplify(domains, constraints);
        match result {
            SolveResult::Stuck => {

                // Heuristic = variable with smallest domain
                // TODO use something smarter here, e.g. most constrained variables
                let mut variables = domains.iter().enumerate()
                    .filter(|(_, domain)| domain.len() > 1)
                    .collect::<Vec<_>>();
                variables
                    .sort_by(|(_, d1), (_, d2)| d1.len().cmp(&d2.len()));
                let mut variables = variables.iter()
                    .map(|(v, _)| *v)
                    .collect::<Vec<_>>();

                // Sort again, looking at most constrained
                let num_constraints = |v| constraints.iter().filter(|c| c.variables().contains(v)).count();
                variables.sort_by(|v1, v2| num_constraints(*v2).cmp(&num_constraints(*v1)));

                for variable in variables.iter() {
                    let domain = domains[*variable];
                    let mut inferred_domain : Domain = domain;
                    for value in domain.iter() {
                        // Guess variable = value and try to solve without branching
                        let mut branch_domains = domains.clone();
                        let mut branch_constraints = constraints.clone();
                        branch_domains[*variable] = Domain::single(value);
                        if self.config.breadcrumbs {
                            self.emit(format!("guess {} = {}", self.variable_name(*variable), value));
                        }
                        // TODO copy !!!
                        let result = self.simplify(&mut branch_domains, &mut branch_constraints);
                        match result {
                            SolveResult::Unsolvable => { inferred_domain.remove(value); },
                            SolveResult::Solved => if self.config.greedy {
                                *domains = branch_domains;
                                *constraints = branch_constraints;
                                return SolveResult::Solved;
                            } else {},
                            _ => {},
                        }
                    }
                    if inferred_domain != domain {
                        if self.config.breadcrumbs {
                            self.emit(format!("{} is {} by guessing", self.variable_name(*variable), inferred_domain));
                        }
                        domains[*variable] = inferred_domain;
                        return self.solve(domains, constraints);
                    }
                }

                return SolveResult::Stuck;
            },
            _ => result,
        }
    }

    // TODO could optimise this for:
    // only solved constraints
    // order of constraints
    // only constraints with dirty variables
    fn simplify(&self, domains: &mut Domains, constraints: &mut Constraints) -> SolveResult {
        loop {
            let mut any_progress = false;
            let mut i = 0;
            while i < constraints.len() {
                let result = constraints[i].check_and_simplify(domains, self);
                match result {
                    SimplifyResult::Unsolvable => {
                        return SolveResult::Unsolvable;
                    },
                    SimplifyResult::Solved => {
                        constraints.swap_remove(i);
                        any_progress = true;
                    },
                    SimplifyResult::Stuck => {
                        i = i + 1;
                    },
                    SimplifyResult::Progress => {
                        i = i + 1;
                        any_progress = true;
                    },
                    SimplifyResult::Rewrite(mut sub_constraints)  => {
                        constraints.swap_remove(i);
                        constraints.append(&mut sub_constraints);
                        any_progress = true;
                    }
                }
            }
            if constraints.is_empty() {
                return SolveResult::Solved;
            } else if !any_progress {
                return SolveResult::Stuck;
            }
        }
    }
}
