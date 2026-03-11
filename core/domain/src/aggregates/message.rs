use chrono::{DateTime, Utc};

use crate::{
    DomainError, DomainResult, Identifiable,
    value_objects::{GameSessionId, MessageId, MessageRole},
};

/// Aggregate
#[derive(Debug)]
pub struct Message {
    id: MessageId,
    session_id: GameSessionId,
    role: MessageRole,
    text: String,
    ts: DateTime<Utc>,
}

impl Message {
    fn validate(text: &str, embedding_id: Option<&str>) -> DomainResult<()> {
        if text.trim().is_empty() {
            return Err(DomainError::InvariantViolation(String::from(
                "Message text must not be empty",
            )));
        }

        if embedding_id.is_some_and(|id| id.trim().is_empty()) {
            return Err(DomainError::InvariantViolation(String::from(
                "Message embedding_id must not be empty when provided",
            )));
        }

        Ok(())
    }

    pub fn new(
        id: MessageId,
        session_id: GameSessionId,
        role: MessageRole,
        text: &str,
        ts: DateTime<Utc>,
        embedding_id: Option<&str>,
    ) -> DomainResult<Self> {
        Self::validate(text, embedding_id)?;

        Ok(Self {
            id,
            session_id,
            role,
            text: text.to_owned(),
            ts,
        })
    }

    pub fn restore(
        id: MessageId,
        session_id: GameSessionId,
        role: MessageRole,
        text: String,
        ts: DateTime<Utc>,
        embedding_id: Option<String>,
    ) -> DomainResult<Self> {
        Self::validate(&text, embedding_id.as_deref())?;

        Ok(Self {
            id,
            session_id,
            role,
            text,
            ts,
        })
    }

    pub fn session_id(&self) -> &GameSessionId {
        &self.session_id
    }

    pub fn role(&self) -> MessageRole {
        self.role
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn ts(&self) -> DateTime<Utc> {
        self.ts
    }
}

impl Identifiable for Message {
    type Id = MessageId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
