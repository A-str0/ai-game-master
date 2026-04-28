use application::ports::{
    ContextObjectRepositoryError, GameSessionRepositoryError, MessageRepositoryError, UnitOfWork,
    UnitOfWorkError, UnitOfWorkFactory, UnitOfWorkResult,
};
use async_trait::async_trait;
use diesel::connection::SimpleConnection;
use domain::aggregates::{ContextObject, GameSession, Message};

use crate::repositories::postgres::{
    connecion::{
        FromPgRepositoryError, PgPool, PgPooledConnection, PgRepositoryError,
        PgRepositoryResultExt, connection,
    },
    context_objects::insert_context_object_conn,
    game_sessions::{insert_game_session_conn, update_game_session_conn},
    messages::insert_message_conn,
};

/// Postgres-backed factory that opens transactional units of work.
#[derive(Clone)]
pub struct PgUnitOfWorkFactory {
    pool: PgPool,
}

impl PgUnitOfWorkFactory {
    /// Creates a factory backed by the supplied Diesel pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FromPgRepositoryError for UnitOfWorkError {
    fn from_pg_repository_error(error: PgRepositoryError) -> Self {
        match error {
            PgRepositoryError::Unavailable { details } => Self::Unavailable { details },
            PgRepositoryError::Internal { details } => Self::Internal { details },
            PgRepositoryError::NotFound { details, .. } => Self::Internal { details },
            PgRepositoryError::Conflict { details, .. } => Self::Internal { details },
        }
    }
}

#[async_trait]
impl UnitOfWorkFactory for PgUnitOfWorkFactory {
    async fn begin(&self) -> UnitOfWorkResult<Box<dyn UnitOfWork>> {
        let mut conn = connection(&self.pool).into_repo()?;
        conn.batch_execute("BEGIN")
            .map_err(|error| UnitOfWorkError::Unavailable {
                details: format!("failed to begin postgres transaction: {error}"),
            })?;

        Ok(Box::new(PgUnitOfWork {
            conn: Some(conn),
            committed: false,
        }))
    }
}

/// Postgres-backed transactional write context.
pub struct PgUnitOfWork {
    conn: Option<PgPooledConnection>,
    committed: bool,
}

impl PgUnitOfWork {
    fn conn_mut(&mut self) -> Result<&mut PgPooledConnection, UnitOfWorkError> {
        self.conn.as_mut().ok_or(UnitOfWorkError::Internal {
            details: String::from("postgres unit of work connection is missing"),
        })
    }
}

impl Drop for PgUnitOfWork {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        if let Some(conn) = self.conn.as_mut() {
            let _ = conn.batch_execute("ROLLBACK");
        }
    }
}

#[async_trait]
impl UnitOfWork for PgUnitOfWork {
    async fn insert_session(
        &mut self,
        session: &GameSession,
    ) -> Result<(), GameSessionRepositoryError> {
        insert_game_session_conn(
            self.conn_mut()
                .map_err(|error| GameSessionRepositoryError::Internal {
                    details: error.to_string(),
                })?,
            session,
        )
    }

    async fn insert_message(&mut self, message: &Message) -> Result<(), MessageRepositoryError> {
        insert_message_conn(
            self.conn_mut()
                .map_err(|error| MessageRepositoryError::Internal {
                    details: error.to_string(),
                })?,
            message,
        )
    }

    async fn insert_context_object(
        &mut self,
        context_object: &ContextObject,
    ) -> Result<(), ContextObjectRepositoryError> {
        insert_context_object_conn(
            self.conn_mut()
                .map_err(|error| ContextObjectRepositoryError::Internal {
                    details: error.to_string(),
                })?,
            context_object,
        )
    }

    async fn update_session(
        &mut self,
        session: &GameSession,
    ) -> Result<(), GameSessionRepositoryError> {
        update_game_session_conn(
            self.conn_mut()
                .map_err(|error| GameSessionRepositoryError::Internal {
                    details: error.to_string(),
                })?,
            session,
        )
    }

    async fn commit(mut self: Box<Self>) -> UnitOfWorkResult<()> {
        let conn = self.conn_mut()?;
        conn.batch_execute("COMMIT")
            .map_err(|error| UnitOfWorkError::Unavailable {
                details: format!("failed to commit postgres transaction: {error}"),
            })?;
        self.committed = true;
        Ok(())
    }
}
