//! Ironstar library - reactive, event-sourced web application framework.
//!
//! # Module Organization
//!
//! - `domain`: Algebraic types (aggregates, events, commands, values, signals)
//! - `application`: Pure business logic (command/query handlers, projections)
//! - `infrastructure`: Effect implementations (SQLite, redb, DuckDB, broadcast)
//! - `presentation`: HTTP handlers and hypertext templates

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
