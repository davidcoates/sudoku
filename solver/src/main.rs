mod bit_set;
mod solver;
mod types;
mod constraint;

use std::rc::Rc;
use std::collections::HashMap;
use std::time::Instant;

use types::*;
use constraint::*;
use bit_set::*;
use solver::*;

use serde_json::json;

struct TrackerImpl {
    variable_index_to_name: Vec<String>,
}

impl Tracker for TrackerImpl {

    fn variable_name(&self, variable: Variable) -> &String {
        return &self.variable_index_to_name[variable];
    }

    fn on_progress(&mut self, comment: String) {
        eprint!("{}\n", comment);
    }

}

fn main() {
    let stdin = std::io::stdin();
    let input = serde_json::from_reader::<std::io::Stdin, serde_json::Value>(stdin).unwrap();

    let mut variable_index_to_name = Vec::new();
    let mut variable_name_to_index: HashMap<String, usize> = HashMap::new();

    let mut domains = Domains::new();
    for (variable, domain_list) in input["domains"].as_object().unwrap() {
        let index = variable_index_to_name.len();
        variable_index_to_name.push(variable.to_string());
        variable_name_to_index.insert(variable.to_string(), index);

        let mut domain = Domain::new();
        for digit in domain_list.as_array().unwrap() {
            domain.insert(usize::try_from(digit.as_u64().unwrap()).unwrap());
        }
        domains.push(domain);
    }

    let mut constraints = Constraints::new();
    for constraint in input["constraints"].as_array().unwrap() {
        let description = constraint["description"].as_str().unwrap().to_string();
        match constraint["type"].as_str().unwrap() {
            "Permutation" => {
                let mut variables = BitSet::new();
                for variable in constraint["variables"].as_array().unwrap() {
                    let variable = variable.as_str().unwrap();
                    variables.insert(variable_name_to_index[variable]);
                }

                let mut domain = Domain::new();
                for digit in constraint["domain"].as_array().unwrap() {
                    domain.insert(usize::try_from(digit.as_u64().unwrap()).unwrap());
                }

                constraints.push(BoxedConstraint::new(Rc::new(Permutation::new(description, variables, domain))));
            }
            "Increasing" => {
                let mut variables = Vec::new();
                for variable in constraint["variables"].as_array().unwrap() {
                    let variable = variable.as_str().unwrap();
                    variables.push(variable_name_to_index[variable]);
                }

                constraints.push(BoxedConstraint::new(Rc::new(Increasing::new(description, variables))));
            }
            _ => panic!("unknown type"),
        }

    }

    let mut tracker = TrackerImpl{
        variable_index_to_name: variable_index_to_name,
    };

    let mut puzzle = Puzzle{ domains, constraints };

    let now = Instant::now();
    let result = puzzle.solve(&mut tracker);
    let elapsed = now.elapsed();

    let result_str = match result {
        Result::Stuck => "stuck",
        Result::Unsolvable => "unsolvable",
        Result::Solved => "solved",
        _ => panic!("unknown result"),
    };

    let mut domains: HashMap<String, serde_json::Value> = HashMap::new();
    for (index, domain) in puzzle.domains.iter().enumerate() {
        let variable = &tracker.variable_index_to_name[index];
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
