//! Secondary ports used by the application layer.
//!
//! Each port describes one integration boundary and keeps its DTOs and error
//! types close to the trait definition.

mod context_object_repository;
mod embedding_port;
mod game_session_repository;
mod memory_extractor_port;
mod message_repository;
mod narrator_port;
mod unit_of_work;
mod user_port;
mod vector_search_port;

pub use context_object_repository::*;
pub use embedding_port::*;
pub use game_session_repository::*;
pub use memory_extractor_port::*;
pub use message_repository::*;
pub use narrator_port::*;
pub use unit_of_work::*;
pub use user_port::*;
pub use vector_search_port::*;
