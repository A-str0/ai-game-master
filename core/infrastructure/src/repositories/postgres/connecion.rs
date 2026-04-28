use diesel::{
    PgConnection,
    r2d2::{self, ConnectionManager},
    result::{DatabaseErrorKind, Error as DieselError},
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use thiserror::Error;

use crate::repositories::unit_of_work;
use crate::repositories::{context_objects, game_sessions, messages};

/// Resource label used when mapping repository errors for context objects.
pub const RESOURCE_CONTEXT_OBJECT: &str = "context_object";
/// Resource label used when mapping repository errors for game sessions.
pub const RESOURCE_GAME_SESSION: &str = "game_session";
/// Resource label used when mapping repository errors for messages.
pub const RESOURCE_MESSAGE: &str = "message";
/// Embedded Diesel migrations bundled with this crate.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Shared Diesel connection pool type used by Postgres repositories.
pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub(crate) type PgPooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

/// Errors returned while initializing the Postgres database facade.
#[derive(Debug, Error)]
pub enum PgDatabaseError {
    /// Connection pool creation failed.
    #[error("failed to build postgres connection pool: {0}")]
    Pool(String),
    /// A connection could not be acquired to run migrations.
    #[error("failed to get postgres connection for migrations: {0}")]
    Connection(String),
    /// Embedded migrations failed to apply.
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

/// Facade that owns the Postgres pool and vends typed repositories.
#[derive(Clone)]
pub struct PgDatabase {
    pool: PgPool,
}

impl PgDatabase {
    /// Creates a database facade, initializes the pool, and runs pending migrations.
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

    /// Creates a facade from an existing pool without running migrations.
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns the underlying Diesel connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Returns a game session repository backed by this database.
    pub fn game_sessions(&self) -> game_sessions::PgGameSessionRepository {
        game_sessions::PgGameSessionRepository::new(self.pool.clone())
    }

    /// Returns a context object repository backed by this database.
    pub fn context_objects(&self) -> context_objects::PgContextObjectRepository {
        context_objects::PgContextObjectRepository::new(self.pool.clone())
    }

    /// Returns a message repository backed by this database.
    pub fn messages(&self) -> messages::PgMessageRepository {
        messages::PgMessageRepository::new(self.pool.clone())
    }

    /// Returns a unit-of-work factory backed by this database.
    pub fn unit_of_work(&self) -> unit_of_work::PgUnitOfWorkFactory {
        unit_of_work::PgUnitOfWorkFactory::new(self.pool.clone())
    }
}

pub(crate) fn connection(pool: &PgPool) -> PgRepositoryResult<PgPooledConnection> {
    pool.get().map_err(|error| PgRepositoryError::Unavailable {
        details: format!("failed to get postgres connection from pool: {error}"),
    })
}
