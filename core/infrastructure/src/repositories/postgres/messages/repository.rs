use application::ports::{MessageRepository, MessageRepositoryError, MessageRepositoryResult};
use diesel::{ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl};
use domain::{aggregates::Message, value_objects::GameSessionId};

use super::{
    models::{MessageRow, NewMessageRow},
    schema::messages::dsl,
};
use crate::repositories::postgres::connecion::{
    DieselResultExt, FromPgRepositoryError, PgPool, PgRepositoryError, PgRepositoryResultExt,
    RESOURCE_MESSAGE, connection,
};

/// Postgres implementation of [`MessageRepository`].
#[derive(Clone)]
pub struct PgMessageRepository {
    pool: PgPool,
}

impl PgMessageRepository {
    /// Creates a repository backed by the supplied Diesel pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FromPgRepositoryError for MessageRepositoryError {
    fn from_pg_repository_error(error: PgRepositoryError) -> Self {
        match error {
            PgRepositoryError::NotFound { details, .. } => Self::NotFound { details },
            PgRepositoryError::Conflict { details, .. } => Self::Conflict { details },
            PgRepositoryError::Unavailable { details } => Self::Unavailable { details },
            PgRepositoryError::Internal { details } => Self::Internal { details },
        }
    }
}

pub(crate) fn insert_message_conn(
    conn: &mut PgConnection,
    message: &Message,
) -> MessageRepositoryResult<()> {
    let row = NewMessageRow::from(message);

    diesel::insert_into(dsl::messages)
        .values(&row)
        .execute(conn)
        .into_repo_diesel(RESOURCE_MESSAGE)?;

    Ok(())
}

pub(crate) fn list_recent_messages_conn(
    conn: &mut PgConnection,
    session_id: &GameSessionId,
    limit: usize,
) -> MessageRepositoryResult<Vec<Message>> {
    if limit == 0 {
        return Ok(Vec::new());
    }

    let rows = dsl::messages
        .filter(dsl::session_id.eq(session_id.0))
        .order((dsl::ts.desc(), dsl::id.desc()))
        .limit(
            i64::try_from(limit).map_err(|_| MessageRepositoryError::Internal {
                details: format!("message list limit {limit} does not fit into i64"),
            })?,
        )
        .load::<MessageRow>(conn)
        .into_repo_diesel(RESOURCE_MESSAGE)?;

    rows.into_iter()
        .map(|row| row.try_into().into_repo())
        .collect()
}

#[async_trait::async_trait]
impl MessageRepository for PgMessageRepository {
    async fn insert(&self, message: &Message) -> MessageRepositoryResult<()> {
        let mut conn = connection(&self.pool).into_repo()?;
        insert_message_conn(&mut conn, message)
    }

    async fn list_recent(
        &self,
        session_id: &GameSessionId,
        limit: usize,
    ) -> MessageRepositoryResult<Vec<Message>> {
        let mut conn = connection(&self.pool).into_repo()?;
        list_recent_messages_conn(&mut conn, session_id, limit)
    }
}
