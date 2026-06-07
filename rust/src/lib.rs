pub mod config;
pub mod app;
pub mod api;
pub mod db;
pub mod services;
pub mod processors;
pub mod websocket;

// Test fixtures and helpers (available for integration tests)
pub mod fixtures;
pub mod helpers;

pub use db::pool::DatabasePool;
