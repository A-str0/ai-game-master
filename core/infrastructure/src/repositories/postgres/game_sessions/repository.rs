use application::ports::{
    GameSessionRepository, GameSessionRepositoryError, GameSessionRepositoryResult,
};
use diesel::{QueryDsl, RunQueryDsl};
use domain::{Identifiable, aggregates::GameSession, value_objects::GameSessionId};

use super::{
    models::{GameSessionChangeset, GameSessionRow, NewGameSessionRow},
    schema::game_sessions::dsl,
};
use crate::repositories::postgres::connecion::{
    PgPool, PgRepositoryError, RESOURCE_GAME_SESSION, connection, map_diesel_error,
};

#[derive(Clone)]
pub struct PgGameSessionRepository {
    pool: PgPool,
}

impl PgGameSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl GameSessionRepository for PgGameSessionRepository {
    async fn create(&self, session: &GameSession) -> GameSessionRepositoryResult<()> {
        let mut conn = connection(&self.pool).map_err(map_repository_error)?;
        let row = NewGameSessionRow::try_from(session).map_err(map_repository_error)?;

        diesel::insert_into(dsl::game_sessions)
            .values(&row)
            .execute(&mut conn)
            .map_err(|error| {
                map_repository_error(map_diesel_error(error, RESOURCE_GAME_SESSION))
            })?;

        Ok(())
    }

    async fn get_by_id(&self, id: &GameSessionId) -> GameSessionRepositoryResult<GameSession> {
        let mut conn = connection(&self.pool).map_err(map_repository_error)?;
        let row = dsl::game_sessions
            .find(id.0)
            .first::<GameSessionRow>(&mut conn)
            .map_err(|error| {
                map_repository_error(map_diesel_error(error, RESOURCE_GAME_SESSION))
            })?;

        row.try_into().map_err(map_repository_error)
    }

    async fn update(&self, session: &GameSession) -> GameSessionRepositoryResult<()> {
        let mut conn = connection(&self.pool).map_err(map_repository_error)?;
        let changes = GameSessionChangeset::try_from(session).map_err(map_repository_error)?;
        let updated_rows = diesel::update(dsl::game_sessions.find(session.id().0))
            .set(&changes)
            .execute(&mut conn)
            .map_err(|error| {
                map_repository_error(map_diesel_error(error, RESOURCE_GAME_SESSION))
            })?;

        if updated_rows == 0 {
            return Err(GameSessionRepositoryError::not_found(
                RESOURCE_GAME_SESSION,
                format!("no rows updated for session {}", session.id().0),
            ));
        }

        Ok(())
    }
}

fn map_repository_error(error: PgRepositoryError) -> GameSessionRepositoryError {
    match error {
        PgRepositoryError::NotFound { resource, details } => {
            GameSessionRepositoryError::NotFound { resource, details }
        }
        PgRepositoryError::Conflict { resource, details } => {
            GameSessionRepositoryError::Conflict { resource, details }
        }
        PgRepositoryError::Unavailable { details } => {
            GameSessionRepositoryError::Unavailable { details }
        }
        PgRepositoryError::Internal { details } => GameSessionRepositoryError::Internal { details },
    }
}
