use application::ports::{MessageRepository, MessageRepositoryError, MessageRepositoryResult};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use domain::{aggregates::Message, value_objects::GameSessionId};

use super::{
    models::{MessageRow, NewMessageRow},
    schema::messages::dsl,
};
use crate::repositories::postgres::connecion::{
    DieselResultExt, FromPgRepositoryError, PgPool, PgRepositoryError, PgRepositoryResultExt,
    RESOURCE_MESSAGE, connection,
};

#[derive(Clone)]
pub struct PgMessageRepository {
    pool: PgPool,
}

impl PgMessageRepository {
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

#[async_trait::async_trait]
impl MessageRepository for PgMessageRepository {
    async fn insert(&self, message: &Message) -> MessageRepositoryResult<()> {
        let mut conn = connection(&self.pool).into_repo()?;
        let row = NewMessageRow::from(message);

        diesel::insert_into(dsl::messages)
            .values(&row)
            .execute(&mut conn)
            .into_repo_diesel(RESOURCE_MESSAGE)?;

        Ok(())
    }

    async fn list_recent(
        &self,
        session_id: &GameSessionId,
        limit: usize,
    ) -> MessageRepositoryResult<Vec<Message>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut conn = connection(&self.pool).into_repo()?;
        let rows = dsl::messages
            .filter(dsl::session_id.eq(session_id.0))
            .order((dsl::ts.desc(), dsl::id.desc()))
            .limit(
                i64::try_from(limit).map_err(|_| MessageRepositoryError::Internal {
                    details: format!("message list limit {limit} does not fit into i64"),
                })?,
            )
            .load::<MessageRow>(&mut conn)
            .into_repo_diesel(RESOURCE_MESSAGE)?;

        rows.into_iter()
            .map(|row| row.try_into().into_repo())
            .collect()
    }
}
