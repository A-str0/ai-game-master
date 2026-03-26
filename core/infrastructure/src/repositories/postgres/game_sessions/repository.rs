use application::ports::{GameSessionRepository, RepoError, RepoResult};
use diesel::{QueryDsl, RunQueryDsl};
use domain::{Identifiable, aggregates::GameSession, value_objects::GameSessionId};

use super::{
    models::{GameSessionChangeset, GameSessionRow, NewGameSessionRow},
    schema::game_sessions::dsl,
};
use crate::repositories::postgres::connecion::{
    PgPool, RESOURCE_GAME_SESSION, connection, map_diesel_error,
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
    async fn create(&self, session: &GameSession) -> RepoResult<()> {
        let mut conn = connection(&self.pool)?;
        let row = NewGameSessionRow::try_from(session)?;

        diesel::insert_into(dsl::game_sessions)
            .values(&row)
            .execute(&mut conn)
            .map_err(|error| map_diesel_error(error, RESOURCE_GAME_SESSION))?;

        Ok(())
    }

    async fn get_by_id(&self, id: &GameSessionId) -> RepoResult<GameSession> {
        let mut conn = connection(&self.pool)?;
        let row = dsl::game_sessions
            .find(id.0)
            .first::<GameSessionRow>(&mut conn)
            .map_err(|error| map_diesel_error(error, RESOURCE_GAME_SESSION))?;

        row.try_into()
    }

    async fn update(&self, session: &GameSession) -> RepoResult<()> {
        let mut conn = connection(&self.pool)?;
        let changes = GameSessionChangeset::try_from(session)?;
        let updated_rows = diesel::update(dsl::game_sessions.find(session.id().0))
            .set(&changes)
            .execute(&mut conn)
            .map_err(|error| map_diesel_error(error, RESOURCE_GAME_SESSION))?;

        if updated_rows == 0 {
            return Err(RepoError::not_found(RESOURCE_GAME_SESSION));
        }

        Ok(())
    }
}
