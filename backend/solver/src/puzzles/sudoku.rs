use crate::types::*;
use crate::constraint::{Constraint, Constraints};
use crate::constraints::*;
use crate::solver::*;

use std::collections::HashMap;
use std::time::Instant;


pub mod api {

use crate::solver::{Config, SolveResult};
use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;

pub type Domain = Vec<usize>;
pub type Domains = HashMap<String, Domain>;
pub type Cell = String;
pub type Cells = Vec<Cell>;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    WhiteKropki,
    BlackKropki,
    X,
    V,
    Thermometer,
    Palindrome,
    Renban,
    Whisper,
}

#[derive(Deserialize, Debug)]
pub struct Constraint {
    pub r#type: ConstraintType,
    pub cells: Cells,
}

#[derive(Deserialize, Debug)]
pub struct GlobalConstraints {
    pub anti_knight: bool,
    pub anti_king: bool,
}

#[derive(Deserialize, Debug)]
pub struct Constraints {
    pub globals: GlobalConstraints,
    pub locals: Vec<Constraint>,
}

#[derive(Deserialize, Debug)]
pub struct Input {
    pub domains: Domains,
    pub constraints: Constraints,
    pub config: Config,
}

#[derive(Serialize, Debug)]
pub struct Output {
    pub domains: Domains,
    pub result: SolveResult,
    pub duration_ms: u128,
}

}

struct Converter {
    pub variable_names: Vec<String>,
    pub constraint_names: Vec<String>,
    pub domains: Domains,
    pub constraints: Constraints,
    variable_name_to_id: HashMap<String, usize>,
}

fn grid_to_variable_id(r: usize, c: usize) -> usize { (r-1)*9 + (c-1) }

impl Converter {

    pub fn new(domains: &api::Domains, constraints: &api::Constraints) -> Result<Self, String> {
        let mut converter = Converter {
            variable_names: Vec::new(),
            constraint_names: Vec::new(),
            domains: Domains::new(),
            constraints: Constraints::new(),
            variable_name_to_id : HashMap::new(),
        };
        converter.convert_domains(domains)?;
        converter.add_sudoku_constraints();
        converter.convert_constraints(constraints)?;
        return Ok(converter);
    }

    fn next_variable_id(&self) -> usize { self.domains.len() }

    fn add_variable(&mut self, name: String, domain: Domain) {
        let id = self.next_variable_id();
        self.variable_name_to_id.insert(name.clone(), id);
        self.variable_names.push(name);
        self.domains.push(domain);
    }

    fn next_constraint_id(&self) -> usize { self.constraints.len() }

    fn add_constraint(&mut self, name: String, constraint: Box<dyn Constraint>) {
        self.constraint_names.push(name);
        self.constraints.push(constraint);
    }

    fn convert_domains(&mut self, domains: &api::Domains) -> Result<(), String> {
        if domains.len() != 9*9 {
            return Err(format!("wrong number of cells"));
        }
        for r in 1..=9 {
            for c in 1..=9 {
                let cell = format!("{}:{}", r, c);
                match domains.get(&cell) {
                    None => {
                        return Err(format!("missing domain for cell({})", cell));
                    },
                    Some(domain) => {
                        let mut converted_domain = Domain::new();
                        for digit in domain.iter() {
                            converted_domain.insert(*digit);
                        }
                        assert_eq!(grid_to_variable_id(r, c), self.next_variable_id());
                        self.add_variable(cell, converted_domain);
                    }
                }
            }
        }
        return Ok(());
    }

    fn convert_cells(&self, cells: &api::Cells) -> Result<Vec<Variable>, String> {
        let mut variables = Vec::new();
        for cell in cells.iter() {
            match self.variable_name_to_id.get(cell) {
                None => return Err(format!("domain missing for cell({})", cell)),
                Some(id) => {
                    variables.push(*id);
                }
            }
        }
        return Ok(variables);
    }

    fn add_move_constraints<const N: usize>(&mut self, name: String, directions: [(isize,isize);N]) {
        for r1 in 1..9 {
            for c1 in 1..9 {
                for (x, y) in directions {
                    let r2 = r1 + x;
                    let c2 = c1 + y;
                    if r2 < 1 || c2 < 1 || r2 > 9 || c2 > 9 {
                        continue;
                    }
                    let mut variables = VariableSet::new();
                    variables.insert(grid_to_variable_id(r1 as usize, c1 as usize));
                    variables.insert(grid_to_variable_id(r2 as usize, c2 as usize));
                    self.add_constraint(name.clone(), Box::new(NotEquals::new(
                        self.next_constraint_id(),
                        variables,
                    )));
                }
            }
        }
    }

    fn convert_constraints(&mut self, constraints: &api::Constraints) -> Result<(), String> {
        for constraint in constraints.locals.iter() {
            self.convert_constraint(constraint)?;
        }
        // TODO get rid of duplicates
        if constraints.globals.anti_knight {
            let directions = [(-2,-1),( 2, 1),( 2,-1),(-2, 1),( 1, 2),(-1, 2),( 1,-2),(-1,-2)];
            self.add_move_constraints("anti_knight".to_string(), directions);
        }
        // TODO get rid of duplicates
        if constraints.globals.anti_king {
            let directions = [(-1,-1),(-1, 0),(-1, 1),( 0,-1),( 0, 1),( 1,-1),( 1, 0),( 1, 1)];
            self.add_move_constraints("anti_king".to_string(), directions);
        }
        return Ok(());
    }

    fn convert_constraint(&mut self, constraint: &api::Constraint) -> Result<(), String> {
        let id = self.next_constraint_id();
        let variable_list = self.convert_cells(&constraint.cells)?;
        let variable_set = VariableSet::from_vec(&variable_list);
        match constraint.r#type {
            api::ConstraintType::WhiteKropki => {
                self.add_constraint("white kropki".to_string(), Box::new(ConsecutiveSet::new(id, variable_set)));
            },
            api::ConstraintType::BlackKropki => {
                self.add_constraint("black kropki".to_string(), Box::new(Ratio::new(id, variable_set, 2)));
            },
            api::ConstraintType::X => {
                self.add_constraint("X".to_string(), Box::new(DistinctSum::new(id, variable_set, 10)));
            },
            api::ConstraintType::V => {
                self.add_constraint("V".to_string(), Box::new(DistinctSum::new(id, variable_set, 5)));
            },
            api::ConstraintType::Thermometer => {
                self.add_constraint("thermometer".to_string(), Box::new(Increasing::new(id, variable_list)));
            },
            api::ConstraintType::Palindrome => {
                for i in 0..(variable_list.len() / 2) {
                    let id = self.next_constraint_id();
                    let variables = vec![variable_list[i], variable_list[variable_list.len() - 1 - i]];
                    self.add_constraint("palindrome".to_string(), Box::new(Increasing::new(id, variables)));
                }
            },
            api::ConstraintType::Renban => {
                self.add_constraint("renban".to_string(), Box::new(ConsecutiveSet::new(id, variable_set)));
            },
            api::ConstraintType::Whisper => {
                self.add_constraint("whisper".to_string(), Box::new(Difference::new(id, variable_list, 5)));
            },
        }
        return Ok(());
    }

    fn add_sudoku_constraints(&mut self) {
        let domain = Domain::from_vec(&vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        for r in 1..=9 {
            let mut variables = VariableSet::new();
            for c in 1..=9 {
                variables.insert(grid_to_variable_id(r, c));
            }
            let id = self.next_constraint_id();
            self.add_constraint(
                format!("sudoku row({})", r),
                Box::new(Permutation::new(id, variables, domain))
            );
        }
        for c in 1..=9 {
            let mut variables = VariableSet::new();
            for r in 1..=9 {
                variables.insert(grid_to_variable_id(r, c));
            }
            let id = self.constraint_names.len();
            self.add_constraint(
                format!("sudoku col({})", c),
                Box::new(Permutation::new(id, variables, domain))
            );
        }
        for box_x in 0..3 {
            for box_y in 0..3 {
                let mut variables = VariableSet::new();
                for i in 0..3 {
                    for j in 0..3 {
                        let r = box_x*3 + i + 1;
                        let c = box_y*3 + j + 1;
                        variables.insert(grid_to_variable_id(r, c));
                    }
                }
                let id = self.constraint_names.len();
                self.add_constraint(
                    format!("sudoku box({})", box_x*3 + box_y + 1),
                    Box::new(Permutation::new(id, variables, domain))
                );
            }
        }
    }

}

pub fn solve(input: api::Input) -> Result<api::Output, String> {

    let converter = Converter::new(&input.domains, &input.constraints)?;

    let mut domains = converter.domains;
    let mut constraints = converter.constraints;

    let solver = Solver{
        variable_names: converter.variable_names,
        constraint_names: converter.constraint_names,
        config: input.config,
    };

    let now = Instant::now();
    let result = solver.solve(&mut domains, &mut constraints);
    let elapsed = now.elapsed();

    let mut output_domains = api::Domains::new();
    for (id, domain) in domains.iter().enumerate() {
        let variable = &solver.variable_name(id);
        output_domains.insert(variable.to_string(), domain.iter().collect());
    }

    return Ok(api::Output{
        domains: output_domains,
        result: result,
        duration_ms: elapsed.as_millis(),
    });
}

#[cfg(test)]
mod tests {

    use super::*;

    fn convert_grid(grid: [[usize; 9]; 9]) -> api::Domains {
        let mut domains = api::Domains::new();
        for r in 1..=9 {
            for c in 1..=9 {
                let cell = format!("{}:{}", r, c);
                if grid[r-1][c-1] == 0 {
                    domains.insert(cell, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
                } else {
                    domains.insert(cell, vec![grid[r-1][c-1]]);
                }
            }
        }
        return domains;
    }

    #[test]
    fn test_simple_sudoku() {

        let config = Config{
            breadcrumbs: false,
            greedy: false,
        };

        let domains = convert_grid([
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

        let input = api::Input {
            domains,
            constraints: api::Constraints {
                globals: api::GlobalConstraints {
                    anti_knight: false,
                    anti_king: false,
                },
                locals: Vec::new(),
            },
            config,
        };

        let expected_domains = convert_grid([
            [9, 3, 4, 1, 7, 2, 6, 5, 8],
            [5, 6, 1, 9, 4, 8, 3, 7, 2],
            [7, 2, 8, 6, 3, 5, 9, 1, 4],
            [4, 1, 7, 2, 6, 9, 5, 8, 3],
            [8, 5, 3, 4, 1, 7, 2, 9, 6],
            [2, 9, 6, 5, 8, 3, 7, 4, 1],
            [1, 4, 9, 7, 2, 6, 8, 3, 5],
            [3, 7, 2, 8, 5, 1, 4, 6, 9],
            [6, 8, 5, 3, 9, 4, 1, 2, 7],
        ]);

        let output = solve(input);

        match output {
            Ok(output) => {
                assert!(matches!(output.result, SolveResult::Solved));
                assert_eq!(output.domains, expected_domains);
            },
            Err(_) => {
                assert!(false);
            }
        }

    }

}
