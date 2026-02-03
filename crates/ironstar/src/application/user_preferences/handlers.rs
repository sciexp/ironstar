//! UserPreferences command handler wiring Decider to EventRepository.
//!
//! This module provides the `handle_user_preferences_command` function that
//! creates an EventSourcedAggregate from the UserPreferences Decider and
//! SQLite event repository, unifying domain and infrastructure errors via
//! `CommandPipelineError`.

use crate::application::error::CommandPipelineError;
use crate::domain::user_preferences::{
    UserPreferencesCommand, UserPreferencesError, UserPreferencesEvent, user_preferences_decider,
};
use crate::infrastructure::event_bus::{EventBus, ZenohEventBus, publish_events_fire_and_forget};
use crate::infrastructure::event_store::SqliteEventRepository;
use fmodel_rust::aggregate::{EventRepository, EventSourcedAggregate};
use std::sync::Arc;

/// Adapter wrapping SqliteEventRepository to map errors to CommandPipelineError.
pub struct UserPreferencesEventRepositoryAdapter {
    inner: Arc<SqliteEventRepository<UserPreferencesCommand, UserPreferencesEvent>>,
}

impl UserPreferencesEventRepositoryAdapter {
    pub fn new(
        inner: Arc<SqliteEventRepository<UserPreferencesCommand, UserPreferencesEvent>>,
    ) -> Self {
        Self { inner }
    }
}

impl EventRepository<UserPreferencesCommand, UserPreferencesEvent, String, CommandPipelineError>
    for UserPreferencesEventRepositoryAdapter
{
    async fn fetch_events(
        &self,
        command: &UserPreferencesCommand,
    ) -> Result<Vec<(UserPreferencesEvent, String)>, CommandPipelineError> {
        self.inner.fetch_events(command).await.map_err(Into::into)
    }

    async fn save(
        &self,
        events: &[UserPreferencesEvent],
    ) -> Result<Vec<(UserPreferencesEvent, String)>, CommandPipelineError> {
        self.inner.save(events).await.map_err(Into::into)
    }

    async fn version_provider(
        &self,
        event: &UserPreferencesEvent,
    ) -> Result<Option<String>, CommandPipelineError> {
        self.inner.version_provider(event).await.map_err(Into::into)
    }
}

/// Handle a UserPreferences command through the EventSourcedAggregate pipeline.
pub async fn handle_user_preferences_command<B: EventBus>(
    event_repository: Arc<SqliteEventRepository<UserPreferencesCommand, UserPreferencesEvent>>,
    event_bus: Option<&B>,
    command: UserPreferencesCommand,
) -> Result<Vec<(UserPreferencesEvent, String)>, CommandPipelineError> {
    let repo_adapter = UserPreferencesEventRepositoryAdapter::new(event_repository);

    let mapped_decider = user_preferences_decider().map_error(|e: &UserPreferencesError| {
        CommandPipelineError::UserPreferences(UserPreferencesError::with_id(
            e.error_id(),
            e.kind().clone(),
        ))
    });

    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    let saved_events = aggregate.handle(&command).await?;

    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

/// Handle a UserPreferences command with Zenoh event bus support.
pub async fn handle_user_preferences_command_zenoh(
    event_repository: Arc<SqliteEventRepository<UserPreferencesCommand, UserPreferencesEvent>>,
    event_bus: Option<&ZenohEventBus>,
    command: UserPreferencesCommand,
) -> Result<Vec<(UserPreferencesEvent, String)>, CommandPipelineError> {
    let repo_adapter = UserPreferencesEventRepositoryAdapter::new(event_repository);

    let mapped_decider = user_preferences_decider().map_error(|e: &UserPreferencesError| {
        CommandPipelineError::UserPreferences(UserPreferencesError::with_id(
            e.error_id(),
            e.kind().clone(),
        ))
    });

    let aggregate = EventSourcedAggregate::new(repo_adapter, mapped_decider);

    let saved_events = aggregate.handle(&command).await?;

    if let Some(bus) = event_bus {
        publish_events_fire_and_forget(bus, &saved_events).await;
    }

    Ok(saved_events)
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::domain::UserId;
    use crate::domain::user_preferences::{PreferencesId, Theme, UserPreferencesErrorKind};
    use crate::infrastructure::event_bus::ZenohEventBus;
    use chrono::Utc;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn create_test_pool() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");

        sqlx::query(include_str!("../../../migrations/001_events.sql"))
            .execute(&pool)
            .await
            .expect("Failed to run migration");

        pool
    }

    const NO_EVENT_BUS: Option<&ZenohEventBus> = None;

    #[tokio::test]
    async fn initialize_succeeds() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = UserPreferencesCommand::InitializePreferences {
            preferences_id: PreferencesId::new(),
            user_id: UserId::new(),
            initialized_at: Utc::now(),
        };

        let result = handle_user_preferences_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_ok());
        let events = result.expect("command should succeed");
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].0,
            UserPreferencesEvent::PreferencesInitialized { .. }
        ));
    }

    #[tokio::test]
    async fn duplicate_initialize_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));
        let user_id = UserId::new();

        let command = UserPreferencesCommand::InitializePreferences {
            preferences_id: PreferencesId::new(),
            user_id,
            initialized_at: Utc::now(),
        };

        let _ = handle_user_preferences_command(Arc::clone(&repo), NO_EVENT_BUS, command)
            .await
            .expect("first initialize should succeed");

        let duplicate = UserPreferencesCommand::InitializePreferences {
            preferences_id: PreferencesId::new(),
            user_id,
            initialized_at: Utc::now(),
        };

        let result = handle_user_preferences_command(repo, NO_EVENT_BUS, duplicate).await;
        assert!(result.is_err());
        match result.expect_err("duplicate should fail") {
            CommandPipelineError::UserPreferences(ref e)
                if *e.kind() == UserPreferencesErrorKind::AlreadyInitialized => {}
            other => panic!("Expected AlreadyInitialized, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn set_theme_without_initialize_fails() {
        let pool = create_test_pool().await;
        let repo = Arc::new(SqliteEventRepository::new(pool));

        let command = UserPreferencesCommand::SetTheme {
            user_id: UserId::new(),
            theme: Theme::Light,
            set_at: Utc::now(),
        };

        let result = handle_user_preferences_command(repo, NO_EVENT_BUS, command).await;
        assert!(result.is_err());
        match result.expect_err("set theme without initialize should fail") {
            CommandPipelineError::UserPreferences(ref e)
                if *e.kind() == UserPreferencesErrorKind::NotInitialized => {}
            other => panic!("Expected NotInitialized, got: {other:?}"),
        }
    }
}
