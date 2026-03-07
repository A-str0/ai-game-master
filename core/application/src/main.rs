use thiserror::Error;

pub mod ports;
pub mod use_cases;

#[derive(Debug, Error)]
pub enum AppError {}

#[tokio::main]
async fn main() {}
