mod bit_set;
mod solver;
mod types;
mod constraint;
mod constraints;

use std::rc::Rc;
use std::collections::HashMap;
use std::time::Instant;

use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use types::*;
use constraints::*;
use constraint::{BoxedConstraint, Constraints};
use solver::Puzzle;


mod api {

use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;

pub type Domain = Vec<usize>;

pub type Domains = HashMap<String, Domain>;

pub type Variable = String;

pub type Variables = Vec<Variable>;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ConstraintParams {
    Permutation { domain: Domain },
    Increasing,
    Equals,
    NotEquals,
    ConsecutiveSet,
    Difference { threshold: usize },
    Ratio { ratio: usize },
    DistinctSum { sum: usize },
    DistinctAntisum { antisums: Vec<usize> },
}

pub use ConstraintParams::*;

#[derive(Deserialize, Debug)]
pub struct Constraint {
    pub variables: Variables,
    pub description: String,
    pub params: ConstraintParams
}

#[derive(Deserialize, Debug)]
pub struct Input {
    pub domains: Domains,
    pub constraints: Vec<Constraint>,
    pub breadcrumbs: bool,
    pub greedy: bool,
    pub max_depth: u64,
}

#[derive(Serialize, Debug)]
pub struct Output {
    pub result: String,
    pub domains: Domains,
    pub duration_ms: u128,
}

}

fn solve(input: api::Input) -> Result<api::Output, String> {

    let mut variable_id_to_name = Vec::new();
    let mut variable_name_to_id: HashMap<String, usize> = HashMap::new();

    let mut domains = Domains::new();
    for (variable, domain_list) in input.domains.iter() {
        let id = variable_id_to_name.len();
        variable_id_to_name.push(variable.to_string());
        variable_name_to_id.insert(variable.to_string(), id);

        let mut domain = Domain::new();
        for digit in domain_list {
            domain.insert(*digit);
        }
        domains.push(domain);
    }

    let mut constraint_id_to_name = Vec::new();
    let mut constraints = Constraints::new();
    for constraint in input.constraints {
        let id = constraint_id_to_name.len();
        constraint_id_to_name.push(constraint.description);
        let mut variables_ordered = Vec::new();
        let mut variables = VariableSet::new();
        for variable in constraint.variables.iter() {
            match variable_name_to_id.get(variable) {
                None => return Err(format!("domain missing for variable({})", variable)),
                Some(variable) => {
                    variables.insert(*variable);
                    variables_ordered.push(*variable);
                }
            }
        }
        match constraint.params {
            api::Permutation { ref domain } => {
                constraints.push(BoxedConstraint::new(Rc::new(Permutation::new(id, variables, Domain::from_vec(domain)))));
            }
            api::Increasing => {
                constraints.push(BoxedConstraint::new(Rc::new(Increasing::new(id, variables_ordered))));
            }
            api::Equals => {
                constraints.push(BoxedConstraint::new(Rc::new(Equals::new(id, variables))));
            }
            api::NotEquals => {
                constraints.push(BoxedConstraint::new(Rc::new(NotEquals::new(id, variables))));
            }
            api::ConsecutiveSet => {
                constraints.push(BoxedConstraint::new(Rc::new(ConsecutiveSet::new(id, variables))));
            }
            api::Difference { threshold } => {
                constraints.push(BoxedConstraint::new(Rc::new(Difference::new(id, variables_ordered, threshold))));
            }
            api::Ratio { ratio } => {
                constraints.push(BoxedConstraint::new(Rc::new(Ratio::new(id, variables, ratio))));
            }
            api::DistinctSum { sum } => {
                constraints.push(BoxedConstraint::new(Rc::new(DistinctSum::new(id, variables, sum))));
            }
            api::DistinctAntisum { ref antisums } => {
                constraints.push(BoxedConstraint::new(Rc::new(DistinctAntisum::new(id, variables, Domain::from_vec(antisums)))));
            }
        }
    }

    let mut reporter = Reporter{
        variable_id_to_name: variable_id_to_name,
        constraint_id_to_name: constraint_id_to_name,
        enabled: input.breadcrumbs,
    };

    let config = Config{
        greedy: input.greedy,
        max_depth: input.max_depth,
    };

    let mut puzzle = Puzzle::new(domains, constraints);

    let now = Instant::now();
    let result = puzzle.solve(&mut reporter, config);
    let elapsed = now.elapsed();

    let result_str = match result {
        constraint::Result::Stuck => "stuck",
        constraint::Result::Unsolvable => "unsolvable",
        constraint::Result::Solved => "solved",
        _ => panic!("unknown result"),
    };

    let mut domains = api::Domains::new();
    for (id, domain) in puzzle.domains.iter().enumerate() {
        let variable = &reporter.variable_name(id);
        domains.insert(variable.to_string(), domain.iter().collect());
    }

    Ok(api::Output {
        result: String::from(result_str),
        domains: domains,
        duration_ms: elapsed.as_millis(),
    })
}

async fn handle_solve_request(req: Request<hyper::body::Incoming>) -> Result<Response<BoxBody<Bytes, Infallible>>, hyper::Error> {
    if req.method() != &Method::POST || req.uri() != "/solve" {
        return Ok(Response::builder()
              .status(StatusCode::NOT_FOUND)
              .body(Empty::new().boxed())
              .unwrap())
    }
    let input_bytes = req.collect().await?.to_bytes();
    let input : serde_json::Result<api::Input> = serde_json::from_slice(&input_bytes);
    match input {
        Err(err) => {
            eprintln!("{:?}", err);
            Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(Empty::new().boxed())
                .unwrap())
        },
        Ok(input) => {
            println!("{:?}", input);
            let output = solve(input);
            match output {
                Err(err) => {
                    eprintln!("{:?}", err);
                    Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(Empty::new().boxed())
                        .unwrap())
                },
                Ok(output) => {
                    println!("{:?}", output);
                    let output_bytes = serde_json::to_vec(&output).unwrap();
                    Ok(Response::new(Full::new(Bytes::from(output_bytes)).boxed()))
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000)); // TODO make the port configurable
    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, hyper::service::service_fn(handle_solve_request))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}
