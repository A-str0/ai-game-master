//! OpenRouter agent adapters implemented with `autoagents`.
mod backstory;
mod llm_config;
mod memory_extractor;
mod models;
mod narrator;
mod parsing;
mod prompts;
mod tools;

pub use backstory::QwenBackstoryAdapter;
pub use memory_extractor::OpenRouterMemoryExtractorAdapter;
pub use narrator::OpenRouterNarratorAdapter;
