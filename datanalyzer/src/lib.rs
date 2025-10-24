/// Datanalyzer - Solana DEX Pool Monitor Library
///
/// This library provides tools for decoding and monitoring Solana DEX pools.

pub mod config;
pub mod dex;
pub mod error;
pub mod models;
pub mod websocket;

// Re-export commonly used types
pub use dex::{create_decoder, DexDecoder, DecoderRegistry, DecoderStats};
pub use error::AppError;
pub use models::{DexType, PoolSnapshot};
pub use websocket::WebSocketManager;
