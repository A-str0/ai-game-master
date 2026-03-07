use chrono::{DateTime, Utc};

use crate::{
    Identifiable,
    value_objects::{ContextObjectId, GameSessionConfig, GameSessionId, GameSessionMode, UserId},
};

/// Aggregate
#[derive(Debug)]
pub struct GameSession {
    id: GameSessionId,
    owner_id: UserId,
    place_id: ContextObjectId, // TODO: move to Character
    session_mode: GameSessionMode,
    config: GameSessionConfig,
}

impl Identifiable for GameSession {
    type Id = GameSessionId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl GameSession {
    pub fn new(owner_id: UserId, session_mode: GameSessionMode, config: GameSessionConfig) -> Self {
        Self {
            id: GameSessionId::new(),
            owner_id,
            place_id: ContextObjectId::new(), // TODO: move to args
            session_mode,
            config,
        }
    }

    pub fn restore(
        id: GameSessionId,
        owner_id: UserId,
        session_mode: GameSessionMode,
        config: GameSessionConfig,
    ) -> Self {
        Self {
            id,
            owner_id,
            place_id: ContextObjectId::new(), // TODO: move to args
            session_mode,
            config,
        }
    }

    pub fn owner_id(&self) -> &UserId {
        &self.owner_id
    }

    pub fn session_mode(&self) -> &GameSessionMode {
        &self.session_mode
    }

    pub fn config(&self) -> &GameSessionConfig {
        &self.config
    }
}

/// Entity
#[derive(Debug)]
pub struct GameSessionMetadata {
    created_ts: DateTime<Utc>,
    last_activity_ts: Option<DateTime<Utc>>,
}

impl GameSessionMetadata {
    pub fn new(created_ts: DateTime<Utc>, last_activity_ts: Option<DateTime<Utc>>) -> Self {
        Self {
            created_ts,
            last_activity_ts,
        }
    }

    pub fn restore(created_ts: DateTime<Utc>, last_activity_ts: Option<DateTime<Utc>>) -> Self {
        Self {
            created_ts,
            last_activity_ts,
        }
    }

    pub fn created_ts(&self) -> DateTime<Utc> {
        self.created_ts
    }

    pub fn last_activity_ts(&self) -> Option<DateTime<Utc>> {
        self.last_activity_ts
    }
}
