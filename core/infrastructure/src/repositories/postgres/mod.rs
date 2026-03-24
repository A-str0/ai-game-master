use application::ports::RepoError;
use diesel::{
    PgConnection,
    r2d2::{self, ConnectionManager},
};
use thiserror::Error;

pub mod game_sessions;
pub mod messages;

pub type PgPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type PgPooledConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Debug, Error)]
pub enum PgDatabaseError {
    #[error("failed to build postgres connection pool: {0}")]
    Pool(String),
}

#[derive(Debug, Error)]
enum PgRepoError {
    #[error(transparent)]
    Query(#[from] diesel::result::Error),
    #[error("postgres pool error: {0}")]
    Pool(String),
}

type PgRepoResult<T> = Result<T, PgRepoError>;
type PgDatabaseResult<T> = Result<T, PgDatabaseError>;

impl From<PgRepoError> for RepoError {
    fn from(error: PgRepoError) -> Self {
        use diesel::result::{DatabaseErrorKind, Error};

        match error {
            PgRepoError::Query(Error::NotFound) => Self::NotFound,
            PgRepoError::Query(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                Self::Conflict
            }
            PgRepoError::Query(Error::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _)) => {
                Self::NotFound
            }
            PgRepoError::Query(_) | PgRepoError::Pool(_) => Self::Unavailable,
        }
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

fn connection(pool: &PgPool) -> PgRepoResult<PgPooledConnection> {
    pool.get()
        .map_err(|error| PgRepoError::Pool(error.to_string()))
}
