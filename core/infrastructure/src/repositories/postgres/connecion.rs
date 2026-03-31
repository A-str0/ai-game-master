use diesel::{
    PgConnection,
    r2d2::{self, ConnectionManager},
    result::{DatabaseErrorKind, Error as DieselError},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use thiserror::Error;

use crate::repositories::{context_objects, game_sessions, messages};

pub const RESOURCE_CONTEXT_OBJECT: &str = "context_object";
pub const RESOURCE_GAME_SESSION: &str = "game_session";
pub const RESOURCE_MESSAGE: &str = "message";
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

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

#[derive(Debug, Error)]
pub(crate) enum PgRepositoryError {
    #[error("{resource} not found: {details}")]
    NotFound {
        resource: &'static str,
        details: String,
    },
    #[error("{resource} already exists: {details}")]
    Conflict {
        resource: &'static str,
        details: String,
    },
    #[error("postgres unavailable: {details}")]
    Unavailable { details: String },
    #[error("postgres returned invalid data: {details}")]
    Internal { details: String },
}

impl PgRepositoryError {
    pub(crate) fn not_found(resource: &'static str, details: impl Into<String>) -> Self {
        Self::NotFound {
            resource,
            details: details.into(),
        }
    }

    pub(crate) fn conflict(resource: &'static str, details: impl Into<String>) -> Self {
        Self::Conflict {
            resource,
            details: details.into(),
        }
    }
}

pub(crate) type PgRepositoryResult<T> = Result<T, PgRepositoryError>;

pub(crate) fn map_diesel_error(error: DieselError, resource: &'static str) -> PgRepositoryError {
    match error {
        DieselError::NotFound => {
            PgRepositoryError::not_found(resource, "diesel query returned no rows")
        }
        DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
            PgRepositoryError::conflict(resource, error.to_string())
        }
        DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _) => {
            PgRepositoryError::not_found(resource, error.to_string())
        }
        _ => PgRepositoryError::Unavailable {
            details: error.to_string(),
        },
    }
}

pub(crate) trait FromPgRepositoryError: Sized {
    fn from_pg_repository_error(error: PgRepositoryError) -> Self;
}

pub(crate) trait PgRepositoryResultExt<T, E> {
    fn into_repo(self) -> Result<T, E>;
}

impl<T, E> PgRepositoryResultExt<T, E> for Result<T, PgRepositoryError>
where
    E: FromPgRepositoryError,
{
    fn into_repo(self) -> Result<T, E> {
        self.map_err(E::from_pg_repository_error)
    }
}

pub(crate) trait DieselResultExt<T, E> {
    fn into_repo_diesel(self, resource: &'static str) -> Result<T, E>;
}

impl<T, E> DieselResultExt<T, E> for Result<T, DieselError>
where
    E: FromPgRepositoryError,
{
    fn into_repo_diesel(self, resource: &'static str) -> Result<T, E> {
        self.map_err(|error| E::from_pg_repository_error(map_diesel_error(error, resource)))
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

    pub fn context_objects(&self) -> context_objects::PgContextObjectRepository {
        context_objects::PgContextObjectRepository::new(self.pool.clone())
    }

    pub fn messages(&self) -> messages::PgMessageRepository {
        messages::PgMessageRepository::new(self.pool.clone())
    }
}

pub(crate) fn connection(pool: &PgPool) -> PgRepositoryResult<PgPooledConnection> {
    pool.get().map_err(|error| PgRepositoryError::Unavailable {
        details: format!("failed to get postgres connection from pool: {error}"),
    })
}
