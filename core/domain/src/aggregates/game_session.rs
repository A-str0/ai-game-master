use chrono::{DateTime, Utc};

use crate::{
    Identifiable,
    aggregates::GameSessionConfig,
    value_objects::{GameSessionId, RngState, UserId},
};

/// Aggregate Root
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

    pub fn owner_id(&self) -> &UserId {
        &self.owner_id
    }

    pub fn config(&self) -> &GameSessionConfig {
        &self.config
    }

    pub fn created_ts(&self) -> DateTime<Utc> {
        self.created_ts
    }

    pub fn updated_ts(&self) -> Option<DateTime<Utc>> {
        self.updated_ts
    }

    pub fn rng_state(&self) -> RngState {
        self.rng_state
    }

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
