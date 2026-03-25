use application::ports::{RepoError, RepoResult};
use diesel::{
    PgConnection,
    r2d2::{self, ConnectionManager},
    result::{DatabaseErrorKind, Error as DieselError},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use thiserror::Error;

pub mod game_sessions;
pub mod messages;

pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const RESOURCE_GAME_SESSION: &str = "game_session";
pub const RESOURCE_MESSAGE: &str = "message";
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Debug, Error)]
pub enum PgDatabaseError {
    #[error("failed to build postgres connection pool: {0}")]
    Pool(String),
    #[error("failed to get postgres connection for migrations: {0}")]
    Connection(String),
    #[error("failed to run postgres migrations: {0}")]
    Migration(String),
}

type PgDatabaseResult<T> = Result<T, PgDatabaseError>;

pub(crate) fn map_diesel_error(error: DieselError, resource: &'static str) -> RepoError {
    match error {
        DieselError::NotFound => RepoError::not_found(resource),
        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
            RepoError::conflict(resource)
        }
        DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
            RepoError::not_found(resource)
        }
        _ => RepoError::Unavailable,
    }
}

#[derive(Clone)]
pub struct PgDatabase {
    pool: PgPool,
}

impl PgDatabase {
    pub fn new(database_url: &str) -> PgDatabaseResult<Self> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = PgPool::builder()
            .build(manager)
            .map_err(|error| PgDatabaseError::Pool(error.to_string()))?;
        let mut connection = pool
            .get()
            .map_err(|error| PgDatabaseError::Connection(error.to_string()))?;
        connection
            .run_pending_migrations(MIGRATIONS)
            .map_err(|error| PgDatabaseError::Migration(error.to_string()))?;

        Ok(Self { pool })
    }

    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub fn game_sessions(&self) -> game_sessions::PgGameSessionRepository {
        game_sessions::PgGameSessionRepository::new(self.pool.clone())
    }

    pub fn messages(&self) -> messages::PgMessageRepository {
        messages::PgMessageRepository::new(self.pool.clone())
    }
}

fn connection(pool: &PgPool) -> RepoResult<PgPooledConnection> {
    pool.get().map_err(|_| RepoError::Unavailable)
}
