//! Todo aggregate state types.
//!
//! State is derived from events and represents the current status of a todo item.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::values::{TodoId, TodoText};

/// Lifecycle status of a todo item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TodoStatus {
    /// Initial state (before any events).
    #[default]
    NotCreated,
    /// Active and not completed.
    Active,
    /// Marked as completed.
    Completed,
    /// Deleted (terminal state).
    Deleted,
}

/// State of a single todo item, derived from events.
///
/// All fields are `Option` because the decider starts empty (before the
/// `Created` event). After `Created`, the fields are guaranteed to be `Some`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TodoState {
    /// Unique identifier (set on Created).
    pub id: Option<TodoId>,
    /// Current text content (set on Created, updated on TextUpdated).
    pub text: Option<TodoText>,
    /// When the todo was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Lifecycle status.
    pub status: TodoStatus,
    /// When it was completed (if applicable).
    pub completed_at: Option<DateTime<Utc>>,
    /// When it was deleted (if applicable).
    pub deleted_at: Option<DateTime<Utc>>,
}

impl TodoState {
    /// Check if the todo exists (has been created).
    #[must_use]
    pub fn exists(&self) -> bool {
        self.status != TodoStatus::NotCreated
    }

    /// Check if the todo is active (not completed, not deleted).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == TodoStatus::Active
    }

    /// Check if the todo is completed.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.status == TodoStatus::Completed
    }

    /// Check if the todo is deleted.
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.status == TodoStatus::Deleted
    }
}
