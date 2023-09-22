#![feature(assert_matches)]

mod bit_set;
mod solver;
mod types;
mod constraint;

use std::rc::Rc;
use std::collections::HashMap;
use std::time::Instant;

use types::*;
use constraint::*;
use solver::*;

use serde_json::json;

struct ReporterImpl {
    variable_id_to_name: Vec<String>,
    constraint_id_to_name: Vec<String>,
    enabled: bool,
}

impl Reporter for ReporterImpl {

    fn variable_name(&self, id: Variable) -> &String {
        &self.variable_id_to_name[id]
    }

    fn constraint_name(&self, id: ConstraintID) -> &String {
        &self.constraint_id_to_name[id]
    }

    fn emit(&mut self, breadcrumb: String) {
        eprint!("{}\n", breadcrumb);
    }

    fn enabled(&self) -> bool {
        self.enabled
    }
}

fn main() {
    let stdin = std::io::stdin();
    let input = serde_json::from_reader::<std::io::Stdin, serde_json::Value>(stdin).unwrap();

    let mut variable_id_to_name = Vec::new();
    let mut variable_name_to_id: HashMap<String, usize> = HashMap::new();

    let mut domains = Domains::new();
    for (variable, domain_list) in input["domains"].as_object().unwrap() {
        let id = variable_id_to_name.len();
        variable_id_to_name.push(variable.to_string());
        variable_name_to_id.insert(variable.to_string(), id);

        let mut domain = Domain::new();
        for digit in domain_list.as_array().unwrap() {
            domain.insert(usize::try_from(digit.as_u64().unwrap()).unwrap());
        }
        domains.push(domain);
    }

    let mut constraint_id_to_name = Vec::new();

    let mut constraints = Constraints::new();
    for constraint in input["constraints"].as_array().unwrap() {
        let id = constraint_id_to_name.len();
        constraint_id_to_name.push(constraint["description"].as_str().unwrap().to_string());
        match constraint["type"].as_str().unwrap() {
            "Permutation" => {
                let mut variables = VariableSet::new();
                for variable in constraint["variables"].as_array().unwrap() {
                    let variable = variable.as_str().unwrap();
                    variables.insert(variable_name_to_id[variable]);
                }

                let mut domain = Domain::new();
                for digit in constraint["domain"].as_array().unwrap() {
                    domain.insert(usize::try_from(digit.as_u64().unwrap()).unwrap());
                }

                constraints.push(BoxedConstraint::new(Rc::new(Permutation::new(id, variables, domain))));
            }
            "Increasing" => {
                let mut variables = Vec::new();
                for variable in constraint["variables"].as_array().unwrap() {
                    let variable = variable.as_str().unwrap();
                    variables.push(variable_name_to_id[variable]);
                }

                constraints.push(BoxedConstraint::new(Rc::new(Increasing::new(id, variables))));
            }
            "Equals" => {
                let mut variables = VariableSet::new();
                for variable in constraint["variables"].as_array().unwrap() {
                    let variable = variable.as_str().unwrap();
                    variables.insert(variable_name_to_id[variable]);
                }
                constraints.push(BoxedConstraint::new(Rc::new(Equals::new(id, variables))));
            }
            "ConsecutiveSet" => {
                let mut variables = VariableSet::new();
                for variable in constraint["variables"].as_array().unwrap() {
                    let variable = variable.as_str().unwrap();
                    variables.insert(variable_name_to_id[variable]);
                }
                constraints.push(BoxedConstraint::new(Rc::new(ConsecutiveSet::new(id, variables))));
            }
            _ => panic!("unknown type"),
        }

    }

    let mut reporter = ReporterImpl{
        variable_id_to_name: variable_id_to_name,
        constraint_id_to_name: constraint_id_to_name,
        enabled: input["breadcrumbs"].as_bool().unwrap(),
    };

    let config = Config{
        strict: input["strict"].as_bool().unwrap(),
        max_depth: input["max_depth"].as_u64().unwrap(),
    };

    let mut puzzle = Puzzle::new(domains, constraints);

    let now = Instant::now();
    let result = puzzle.solve(&mut reporter, config);
    let elapsed = now.elapsed();

    let result_str = match result {
        Result::Stuck => "stuck",
        Result::Unsolvable => "unsolvable",
        Result::Solved => "solved",
        _ => panic!("unknown result"),
    };

    let mut domains: HashMap<String, serde_json::Value> = HashMap::new();
    for (id, domain) in puzzle.domains.iter().enumerate() {
        let variable = &reporter.variable_id_to_name[id];
        let domain = domain.iter().map(|x| json!(x)).collect::<Vec<serde_json::Value>>();
        domains.insert(variable.to_string(), json!(domain));
    }

    let output = json!({
        "result" : json!(result_str),
        "domains" : json!(domains),
        "duration_ms" : json!(elapsed.as_millis()),
    });

    serde_json::to_writer(std::io::stdout(), &output).ok();
}
