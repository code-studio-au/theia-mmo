//! Small local walking protocol, not an MMO service stack.
pub mod client;
pub mod server;
pub mod simulation;
pub mod tls;
pub mod wire;
pub mod world;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
