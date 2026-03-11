use application::ports::{RepoError, RepoResult};
use chrono::{DateTime, Utc};
use diesel::sql_types::SqlType;
use diesel::{Identifiable, Insertable, Queryable};
use diesel_derive_enum::DbEnum;
use domain::{
    Identifiable as DomainIdentifiable,
    aggregates::Message,
    value_objects::{GameSessionId, MessageId, MessageRole},
};
use uuid::Uuid;

use super::schema::messages;

#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
#[PgType = "MessageRole"]
pub enum MessageRoleDb {
    Player,
    Gm,
    System,
}

impl From<MessageRole> for MessageRoleDb {
    fn from(value: MessageRole) -> Self {
        match value {
            MessageRole::Player => Self::Player,
            MessageRole::Gm => Self::Gm,
            MessageRole::System => Self::System,
        }
    }
}

impl From<MessageRoleDb> for MessageRole {
    fn from(value: MessageRoleDb) -> Self {
        match value {
            MessageRoleDb::Player => Self::Player,
            MessageRoleDb::Gm => Self::Gm,
            MessageRoleDb::System => Self::System,
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct MessageRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRoleDb,
    pub text: String,
    pub ts: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewMessageRow {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRoleDb,
    pub text: String,
    pub ts: DateTime<Utc>,
}

impl TryFrom<MessageRow> for Message {
    type Error = RepoError;

    fn try_from(row: MessageRow) -> RepoResult<Self> {
        Message::restore(
            MessageId(row.id),
            GameSessionId(row.session_id),
            row.role.into(),
            row.text,
            row.ts,
            None,
        )
        .map_err(|_| RepoError::Unavailable)
    }
}

impl From<&Message> for NewMessageRow {
    fn from(message: &Message) -> Self {
        Self {
            id: message.id().0,
            session_id: message.session_id().0,
            role: message.role().into(),
            text: message.text().to_owned(),
            ts: message.ts(),
        }
    }
}
