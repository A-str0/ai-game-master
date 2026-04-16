//! OpenRouter agent adapters implemented with `autoagents`.

mod llm_config;
mod memory_extractor;
mod models;
mod narrator;
mod parsing;
mod prompts;
mod tools;

pub use memory_extractor::OpenRouterMemoryExtractorAdapter;
pub use narrator::OpenRouterNarratorAdapter;
