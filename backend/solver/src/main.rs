mod bit_set;
mod solver;
mod types;
mod constraint;
mod constraints;
mod puzzles;

use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use puzzles::*;

mod api {

use serde::Deserialize;
use serde::Serialize;
use crate::puzzles;

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Input {
    Sudoku(puzzles::sudoku::api::Input)
}

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output {
    Sudoku(puzzles::sudoku::api::Output)
}

}

fn solve(input: api::Input) -> Result<api::Output, String> {
    match input {
        api::Input::Sudoku(input_data) => match sudoku::solve(input_data) {
            Ok(output_data) => Ok(api::Output::Sudoku(output_data)),
            Err(e)          => Err(e),
        }
    }
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
