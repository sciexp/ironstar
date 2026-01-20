//! Ironstar library - reactive, event-sourced web application framework.
//!
//! # Module Organization
//!
//! - `common`: Shared types across all layers (error codes)
//! - `config`: Application configuration from environment variables
//! - `domain`: Algebraic types (aggregates, events, commands, values, signals)
//! - `application`: Pure business logic (command/query handlers, projections)
//! - `infrastructure`: Effect implementations (SQLite, redb, DuckDB, broadcast)
//! - `presentation`: HTTP handlers and hypertext templates
//! - `state`: Application state container with FromRef implementations

pub mod application;
pub mod common;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod state;
