use crate::repositories::postgres::connecion::{PgRepositoryError, PgRepositoryResult};
use chrono::{DateTime, Utc};
use diesel::sql_types::SqlType;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable};
use diesel_derive_enum::DbEnum;
use domain::aggregates::GameSessionConfig;
use domain::{
    Identifiable as DomainIdentifiable,
    aggregates::GameSession,
    value_objects::{GameSessionId, GameSessionMode, RngState, ScoringWeights, UserId},
};
use uuid::Uuid;

use super::schema::game_sessions;

#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
#[PgType = "SessionMode"]
#[DbValueStyle = "PascalCase"]
pub enum SessionModeDb {
    Solo,
    Multi,
}

impl From<GameSessionMode> for SessionModeDb {
    fn from(value: GameSessionMode) -> Self {
        match value {
            GameSessionMode::Solo => Self::Solo,
            GameSessionMode::Multi => Self::Multi,
        }
    }
}

impl From<SessionModeDb> for GameSessionMode {
    fn from(value: SessionModeDb) -> Self {
        match value {
            SessionModeDb::Solo => Self::Solo,
            SessionModeDb::Multi => Self::Multi,
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = game_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct GameSessionRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub retrivial_k: i16,
    pub memory_budget: i32,
    pub session_mode: SessionModeDb,
    pub created_ts: DateTime<Utc>,
    pub last_activity_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = game_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewGameSessionRow {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub retrivial_k: i16,
    pub memory_budget: i32,
    pub session_mode: SessionModeDb,
    pub created_ts: DateTime<Utc>,
    pub last_activity_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = game_sessions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct GameSessionChangeset {
    pub owner_id: Uuid,
    pub retrivial_k: i16,
    pub memory_budget: i32,
    pub session_mode: SessionModeDb,
    pub last_activity_ts: Option<DateTime<Utc>>,
}

impl TryFrom<GameSessionRow> for GameSession {
    type Error = PgRepositoryError;

    fn try_from(row: GameSessionRow) -> PgRepositoryResult<Self> {
        let retrivial_k =
            u8::try_from(row.retrivial_k).map_err(|_| PgRepositoryError::Internal {
                details: format!("retrivial_k {} does not fit into u8", row.retrivial_k),
            })?;
        let memory_budget =
            u32::try_from(row.memory_budget).map_err(|_| PgRepositoryError::Internal {
                details: format!("memory_budget {} does not fit into u32", row.memory_budget),
            })?;
        let config = GameSessionConfig::restore(
            retrivial_k,
            memory_budget,
            ScoringWeights::default(),
            row.session_mode.into(),
        )
        .map_err(|error| PgRepositoryError::Internal {
            details: format!("game_session row violates domain invariants: {error}"),
        })?;

        Ok(GameSession::restore(
            GameSessionId(row.id),
            UserId(row.owner_id),
            config,
            RngState::default(),
            row.created_ts,
            row.last_activity_ts,
        ))
    }
}

impl TryFrom<&GameSession> for NewGameSessionRow {
    type Error = PgRepositoryError;

    fn try_from(session: &GameSession) -> PgRepositoryResult<Self> {
        Ok(Self {
            id: session.id().0,
            owner_id: session.owner_id().0,
            retrivial_k: i16::from(session.config().retrivial_k()),
            memory_budget: i32::try_from(session.config().memory_budget()).map_err(|_| {
                PgRepositoryError::Internal {
                    details: format!(
                        "memory_budget {} does not fit into i32",
                        session.config().memory_budget()
                    ),
                }
            })?,
            session_mode: (*session.config().session_mode()).into(),
            created_ts: session.created_ts(),
            last_activity_ts: session.updated_ts(),
        })
    }
}

impl TryFrom<&GameSession> for GameSessionChangeset {
    type Error = PgRepositoryError;

    fn try_from(session: &GameSession) -> PgRepositoryResult<Self> {
        Ok(Self {
            owner_id: session.owner_id().0,
            retrivial_k: i16::from(session.config().retrivial_k()),
            memory_budget: i32::try_from(session.config().memory_budget()).map_err(|_| {
                PgRepositoryError::Internal {
                    details: format!(
                        "memory_budget {} does not fit into i32",
                        session.config().memory_budget()
                    ),
                }
            })?,
            session_mode: (*session.config().session_mode()).into(),
            last_activity_ts: session.updated_ts(),
        })
    }
}
