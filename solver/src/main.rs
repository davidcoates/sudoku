mod bit_set;
mod solver;
mod types;
mod constraint;
mod constraints;

use std::rc::Rc;
use std::collections::HashMap;
use std::time::Instant;

use types::*;
use constraint::*;
use solver::*;
use constraints::*;

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

    fn emit(&self, breadcrumb: String) {
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
            domain.insert(digit.as_u64().unwrap() as usize);
        }
        domains.push(domain);
    }

    let mut constraint_id_to_name = Vec::new();

    let mut constraints = Constraints::new();
    for constraint in input["constraints"].as_array().unwrap() {
        let id = constraint_id_to_name.len();
        constraint_id_to_name.push(constraint["description"].as_str().unwrap().to_string());
        let mut variables_ordered = Vec::new();
        let mut variables = VariableSet::new();
        for variable in constraint["variables"].as_array().unwrap() {
            let variable = variable.as_str().unwrap();
            variables.insert(variable_name_to_id[variable]);
            variables_ordered.push(variable_name_to_id[variable])
        }
        match constraint["type"].as_str().unwrap() {
            "Permutation" => {
                let mut domain = Domain::new();
                for digit in constraint["domain"].as_array().unwrap() {
                    domain.insert(digit.as_u64().unwrap() as usize);
                }

                constraints.push(BoxedConstraint::new(Rc::new(Permutation::new(id, variables, domain))));
            }
            "Increasing" => {
                constraints.push(BoxedConstraint::new(Rc::new(Increasing::new(id, variables_ordered))));
            }
            "Equals" => {
                constraints.push(BoxedConstraint::new(Rc::new(Equals::new(id, variables))));
            }
            "NotEquals" => {
                constraints.push(BoxedConstraint::new(Rc::new(NotEquals::new(id, variables))));
            }
            "ConsecutiveSet" => {
                constraints.push(BoxedConstraint::new(Rc::new(ConsecutiveSet::new(id, variables))));
            }
            "Difference" => {
                let threshold = constraint["threshold"].as_u64().unwrap() as usize;
                constraints.push(BoxedConstraint::new(Rc::new(Difference::new(id, variables_ordered, threshold))));
            }
            "Ratio" => {
                let ratio = constraint["ratio"].as_u64().unwrap() as usize;
                constraints.push(BoxedConstraint::new(Rc::new(Ratio::new(id, variables, ratio))));
            }
            "DistinctSum" => {
                let sum = constraint["sum"].as_u64().unwrap() as usize;
                constraints.push(BoxedConstraint::new(Rc::new(DistinctSum::new(id, variables, sum))));
            }
            "DistinctAntisum" => {
                let mut antisums = Domain::new();
                for digit in constraint["antisums"].as_array().unwrap() {
                    antisums.insert(digit.as_u64().unwrap() as usize);
                }
                constraints.push(BoxedConstraint::new(Rc::new(DistinctAntisum::new(id, variables, antisums))));
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
        greedy: input["greedy"].as_bool().unwrap(),
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
