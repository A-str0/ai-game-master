use chrono::{DateTime, Utc};

use crate::{
    Identifiable,
    value_objects::{GameSessionConfig, GameSessionId, UserId},
};

/// Aggregate root
#[derive(Debug)]
pub struct GameSession {
    id: GameSessionId,
    owner_id: UserId,
    config: GameSessionConfig,
    // TODO: rng_state: RngState
    created_ts: DateTime<Utc>,
    last_activity_ts: Option<DateTime<Utc>>,
}

impl Identifiable for GameSession {
    type Id = GameSessionId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl GameSession {
    pub fn new(
        id: GameSessionId,
        owner_id: UserId,
        config: GameSessionConfig,
        created_ts: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            owner_id,
            config,
            created_ts,
            last_activity_ts: None,
        }
    }

    pub fn restore(
        id: GameSessionId,
        owner_id: UserId,
        config: GameSessionConfig,
        created_ts: DateTime<Utc>,
        last_activity_ts: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            owner_id,
            config,
            created_ts,
            last_activity_ts,
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

    pub fn last_activity_ts(&self) -> Option<DateTime<Utc>> {
        self.last_activity_ts
    }

    pub fn record_activity(&mut self, at: DateTime<Utc>) {
        self.last_activity_ts = Some(at);
    }
}
