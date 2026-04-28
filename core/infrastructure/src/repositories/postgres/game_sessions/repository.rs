use application::ports::{
    GameSessionRepository, GameSessionRepositoryError, GameSessionRepositoryResult,
};
use diesel::{PgConnection, QueryDsl, RunQueryDsl};
use domain::{Identifiable, aggregates::GameSession, value_objects::GameSessionId};

use super::{
    models::{GameSessionChangeset, GameSessionRow, NewGameSessionRow},
    schema::game_sessions::dsl,
};
use crate::repositories::postgres::connecion::{
    DieselResultExt, FromPgRepositoryError, PgPool, PgRepositoryError, PgRepositoryResultExt,
    RESOURCE_GAME_SESSION, connection,
};

/// Postgres implementation of [`GameSessionRepository`].
#[derive(Clone)]
pub struct PgGameSessionRepository {
    pool: PgPool,
}

impl PgGameSessionRepository {
    /// Creates a repository backed by the supplied Diesel pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FromPgRepositoryError for GameSessionRepositoryError {
    fn from_pg_repository_error(error: PgRepositoryError) -> Self {
        match error {
            PgRepositoryError::NotFound { details, .. } => Self::NotFound { details },
            PgRepositoryError::Conflict { details, .. } => Self::Conflict { details },
            PgRepositoryError::Unavailable { details } => Self::Unavailable { details },
            PgRepositoryError::Internal { details } => Self::Internal { details },
        }
    }
}

pub(crate) fn insert_game_session_conn(
    conn: &mut PgConnection,
    session: &GameSession,
) -> GameSessionRepositoryResult<()> {
    let row = NewGameSessionRow::try_from(session).into_repo()?;

    diesel::insert_into(dsl::game_sessions)
        .values(&row)
        .execute(conn)
        .into_repo_diesel(RESOURCE_GAME_SESSION)?;

    Ok(())
}

pub(crate) fn get_game_session_by_id_conn(
    conn: &mut PgConnection,
    id: &GameSessionId,
) -> GameSessionRepositoryResult<GameSession> {
    let row = dsl::game_sessions
        .find(id.0)
        .first::<GameSessionRow>(conn)
        .into_repo_diesel(RESOURCE_GAME_SESSION)?;

    row.try_into().into_repo()
}

pub(crate) fn update_game_session_conn(
    conn: &mut PgConnection,
    session: &GameSession,
) -> GameSessionRepositoryResult<()> {
    let changes = GameSessionChangeset::try_from(session).into_repo()?;
    let updated_rows = diesel::update(dsl::game_sessions.find(session.id().0))
        .set(&changes)
        .execute(conn)
        .into_repo_diesel(RESOURCE_GAME_SESSION)?;

    if updated_rows == 0 {
        return Err(GameSessionRepositoryError::NotFound {
            details: format!("no rows updated for session {}", session.id().0),
        });
    }

    Ok(())
}

#[async_trait::async_trait]
impl GameSessionRepository for PgGameSessionRepository {
    async fn insert(&self, session: &GameSession) -> GameSessionRepositoryResult<()> {
        let mut conn = connection(&self.pool).into_repo()?;
        insert_game_session_conn(&mut conn, session)
    }

    async fn get_by_id(&self, id: &GameSessionId) -> GameSessionRepositoryResult<GameSession> {
        let mut conn = connection(&self.pool).into_repo()?;
        get_game_session_by_id_conn(&mut conn, id)
    }

    async fn update(&self, session: &GameSession) -> GameSessionRepositoryResult<()> {
        let mut conn = connection(&self.pool).into_repo()?;
        update_game_session_conn(&mut conn, session)
    }
}
