mod bit_set;
mod solver;
mod domain;
mod constraint;

use std::rc::Rc;
use std::collections::HashMap;

use domain::*;
use constraint::*;
use bit_set::*;
use solver::*;

use serde_json::json;

fn main() {
    let stdin = std::io::stdin();
    let input = serde_json::from_reader::<std::io::Stdin, serde_json::Value>(stdin).unwrap();

    let mut index_to_variable = Vec::new();
    let mut variable_to_index: HashMap<String, usize> = HashMap::new();

    let mut domains = Domains::new();
    for (variable, domain_list) in input["domains"].as_object().unwrap() {
        let index = index_to_variable.len();
        index_to_variable.push(variable);
        variable_to_index.insert(variable.to_string(), index);

        let mut domain = Domain::new();
        for digit in domain_list.as_array().unwrap() {
            domain.insert(usize::try_from(digit.as_u64().unwrap()).unwrap());
        }
        domains.push(domain);
    }

    let mut constraints = Constraints::new();
    for constraint in input["constraints"].as_array().unwrap() {
        match constraint["type"].as_str().unwrap() {
            "Permutation" => {
                let mut variables = BitSet::new();
                for variable in constraint["variables"].as_array().unwrap() {
                    let variable = variable.as_str().unwrap();
                    variables.insert(variable_to_index[variable]);
                }

                let mut domain = Domain::new();
                for digit in constraint["domain"].as_array().unwrap() {
                    domain.insert(usize::try_from(digit.as_u64().unwrap()).unwrap());
                }

                constraints.push(BoxedConstraint::new(Rc::new(Permutation::new(variables, domain))));
            }
            "Increasing" => {
                let mut variables = BitSet::new();
                for variable in constraint["variables"].as_array().unwrap() {
                    let variable = variable.as_str().unwrap();
                    variables.insert(variable_to_index[variable]);
                }

                constraints.push(BoxedConstraint::new(Rc::new(Increasing::new(variables))));
            }
            _ => panic!("unknown type"),
        }

    }

    let mut puzzle = Puzzle{ domains, constraints };
    let result = puzzle.solve();

    let result_str = match result {
        Result::Stuck => "stuck",
        Result::Unsolvable => "unsolvable",
        Result::Solved => "solved",
        _ => panic!("unknown result"),
    };

    let mut domains: HashMap<String, serde_json::Value> = HashMap::new();
    for (index, domain) in puzzle.domains.iter().enumerate() {
        let variable = index_to_variable[index];
        let domain = domain.bit_set().iter().map(|x| json!(x)).collect::<Vec<serde_json::Value>>();
        domains.insert(variable.to_string(), json!(domain));
    }

    let output = json!({
        "result" : json!(result_str),
        "domains" : json!(domains),
    });

    serde_json::to_writer(std::io::stdout(), &output).ok();
}
