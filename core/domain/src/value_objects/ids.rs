use derive_more::{From, Into};
use uuid::Uuid;

/// Strongly typed identifier for a [`crate::aggregates::GameSession`].
#[derive(Debug, From, Into, PartialEq, Eq, Clone, Copy, Hash)]
pub struct GameSessionId(pub Uuid);

impl GameSessionId {
    /// Returns the nil UUID wrapper.
    ///
    /// This is mainly useful in tests or as a placeholder before a real ID is
    /// assigned.
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

/// Strongly typed identifier for a [`crate::aggregates::ContextObject`].
#[derive(Debug, From, Into, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ContextObjectId(pub Uuid);

impl ContextObjectId {
    /// Returns the nil UUID wrapper.
    ///
    /// This is mainly useful in tests or as a placeholder before a real ID is
    /// assigned.
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

/// Strongly typed identifier for an authenticated user.
#[derive(Debug, From, Into, PartialEq, Eq, Clone, Copy, Hash)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Returns the nil UUID wrapper.
    ///
    /// This is mainly useful in tests or as a placeholder before a real ID is
    /// assigned.
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

/// Strongly typed identifier for a [`crate::aggregates::Message`].
#[derive(Debug, From, Into, PartialEq, Eq, Clone, Copy, Hash)]
pub struct MessageId(pub Uuid);

impl MessageId {
    /// Returns the nil UUID wrapper.
    ///
    /// This is mainly useful in tests or as a placeholder before a real ID is
    /// assigned.
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}
