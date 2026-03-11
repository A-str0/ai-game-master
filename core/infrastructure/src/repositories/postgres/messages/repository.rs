use application::ports::{MessageRepository, RepoResult};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use domain::{aggregates::Message, value_objects::GameSessionId};

use super::{
    models::{MessageRow, NewMessageRow},
    schema::messages::dsl,
};
use crate::repositories::postgres::{PgPool, PgRepoError, connection};

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
    async fn create(&self, message: &Message) -> RepoResult<()> {
        let mut conn = connection(&self.pool)?;
        let row = NewMessageRow::from(message);

        diesel::insert_into(dsl::messages)
            .values(&row)
            .execute(&mut conn)
            .map_err(PgRepoError::from)?;

        Ok(())
    }

    async fn list_recent(
        &self,
        session_id: &GameSessionId,
        limit: usize,
    ) -> RepoResult<Vec<Message>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut conn = connection(&self.pool)?;
        let rows = dsl::messages
            .filter(dsl::session_id.eq(session_id.0))
            .order((dsl::ts.desc(), dsl::id.desc()))
            .limit(i64::try_from(limit).map_err(|_| application::ports::RepoError::Unavailable)?)
            .load::<MessageRow>(&mut conn)
            .map_err(PgRepoError::from)?;

        rows.into_iter().map(TryInto::try_into).collect()
    }
}
