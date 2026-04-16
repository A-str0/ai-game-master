use chrono::{DateTime, Utc};

use crate::{
    DomainError, DomainResult, Identifiable,
    value_objects::{GameSessionId, MessageId, MessageRole},
};

/// Aggregate root that captures one canonical in-session message.
#[derive(Debug)]
pub struct Message {
    id: MessageId,
    session_id: GameSessionId,
    role: MessageRole,
    text: String,
    ts: DateTime<Utc>,
}

impl Message {
    fn validate(text: &str) -> DomainResult<()> {
        if text.trim().is_empty() {
            return Err(DomainError::InvariantViolation(String::from(
                "Message text must not be empty",
            )));
        }

        Ok(())
    }

    /// Creates a new validated message entry.
    pub fn new(
        id: MessageId,
        session_id: GameSessionId,
        role: MessageRole,
        text: &str,
        ts: DateTime<Utc>,
    ) -> DomainResult<Self> {
        Self::validate(text)?;

        Ok(Self {
            id,
            session_id,
            role,
            text: text.to_owned(),
            ts,
        })
    }

    /// Restores a message from persisted state.
    pub fn restore(
        id: MessageId,
        session_id: GameSessionId,
        role: MessageRole,
        text: String,
        ts: DateTime<Utc>,
    ) -> DomainResult<Self> {
        Self::validate(&text)?;

        Ok(Self {
            id,
            session_id,
            role,
            text,
            ts,
        })
    }

    /// Returns the session that this message belongs to.
    pub fn session_id(&self) -> &GameSessionId {
        &self.session_id
    }

    /// Returns whether the message came from the player, GM, or system.
    pub fn role(&self) -> MessageRole {
        self.role
    }

    /// Returns the raw message text stored in history.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns when the message was created.
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
