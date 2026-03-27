use application::ports::{MessageRepository, MessageRepositoryError, MessageRepositoryResult};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use domain::{aggregates::Message, value_objects::GameSessionId};

use super::{
    models::{MessageRow, NewMessageRow},
    schema::messages::dsl,
};
use crate::repositories::postgres::connecion::{
    PgPool, PgRepositoryError, RESOURCE_MESSAGE, connection, map_diesel_error,
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

#[async_trait::async_trait]
impl MessageRepository for PgMessageRepository {
    async fn create(&self, message: &Message) -> MessageRepositoryResult<()> {
        let mut conn = connection(&self.pool).map_err(map_repository_error)?;
        let row = NewMessageRow::from(message);

        diesel::insert_into(dsl::messages)
            .values(&row)
            .execute(&mut conn)
            .map_err(|error| map_repository_error(map_diesel_error(error, RESOURCE_MESSAGE)))?;

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

        let mut conn = connection(&self.pool).map_err(map_repository_error)?;
        let rows = dsl::messages
            .filter(dsl::session_id.eq(session_id.0))
            .order((dsl::ts.desc(), dsl::id.desc()))
            .limit(
                i64::try_from(limit).map_err(|_| MessageRepositoryError::Internal {
                    details: format!("message list limit {limit} does not fit into i64"),
                })?,
            )
            .load::<MessageRow>(&mut conn)
            .map_err(|error| map_repository_error(map_diesel_error(error, RESOURCE_MESSAGE)))?;

        rows.into_iter()
            .map(|row| row.try_into().map_err(map_repository_error))
            .collect()
    }
}

fn map_repository_error(error: PgRepositoryError) -> MessageRepositoryError {
    match error {
        PgRepositoryError::NotFound { resource, details } => {
            MessageRepositoryError::NotFound { resource, details }
        }
        PgRepositoryError::Conflict { resource, details } => {
            MessageRepositoryError::Conflict { resource, details }
        }
        PgRepositoryError::Unavailable { details } => {
            MessageRepositoryError::Unavailable { details }
        }
        PgRepositoryError::Internal { details } => MessageRepositoryError::Internal { details },
    }
}
