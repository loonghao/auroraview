//! AI Provider abstraction using genai crate
//!
//! This module provides a unified interface to multiple AI providers
//! using the genai crate (https://github.com/jeremychone/rust-genai)
//!
//! Supported providers:
//! - OpenAI (GPT-4o, GPT-4, etc.)
//! - Anthropic (Claude 3.5, 3.7, etc.)
//! - Google Gemini
//! - DeepSeek
//! - Ollama (local models)
//! - Groq, xAI, Cohere, and more

mod types;
mod wrapper;

pub use types::*;
pub use wrapper::*;
