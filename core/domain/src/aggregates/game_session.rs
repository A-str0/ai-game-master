use chrono::{DateTime, Utc};

use crate::{
    Identifiable,
    aggregates::GameSessionConfig,
    value_objects::{GameSessionId, RngState, UserId},
};

/// Aggregate root that represents a single game session owned by one user.
#[derive(Debug)]
pub struct GameSession {
    id: GameSessionId,
    owner_id: UserId,
    config: GameSessionConfig,
    rng_state: RngState,
    created_ts: DateTime<Utc>,
    updated_ts: Option<DateTime<Utc>>,
}

impl GameSession {
    /// Creates a new session with no recorded activity yet.
    pub fn new(
        id: GameSessionId,
        owner_id: UserId,
        config: GameSessionConfig,
        rng_state: RngState,
        created_ts: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            owner_id,
            config,
            rng_state,
            created_ts,
            updated_ts: None,
        }
    }

    /// Restores a session from persisted state.
    pub fn restore(
        id: GameSessionId,
        owner_id: UserId,
        config: GameSessionConfig,
        rng_state: RngState,
        created_ts: DateTime<Utc>,
        updated_ts: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            owner_id,
            config,
            rng_state,
            created_ts,
            updated_ts,
        }
    }

    /// Returns the user who owns and is authorized to access this session.
    pub fn owner_id(&self) -> &UserId {
        &self.owner_id
    }

    /// Returns the configuration that drives retrieval and prompting behavior.
    pub fn config(&self) -> &GameSessionConfig {
        &self.config
    }

    /// Returns when the session was created.
    pub fn created_ts(&self) -> DateTime<Utc> {
        self.created_ts
    }

    /// Returns the timestamp of the latest recorded activity, if any.
    pub fn updated_ts(&self) -> Option<DateTime<Utc>> {
        self.updated_ts
    }

    /// Returns the deterministic RNG cursor used by downstream agents.
    pub fn rng_state(&self) -> RngState {
        self.rng_state
    }

    /// Marks the session as having observed activity at the provided timestamp.
    pub fn mark_activity(&mut self, ts: DateTime<Utc>) {
        self.updated_ts = Some(ts);
    }
}

impl Identifiable for GameSession {
    type Id = GameSessionId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
