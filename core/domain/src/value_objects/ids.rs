use derive_more::{From, Into};
use uuid::Uuid;

#[derive(Debug, From, Into, PartialEq, Eq, Clone, Copy, Hash)]
pub struct GameSessionId(pub Uuid);

impl GameSessionId {
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

#[derive(Debug, From, Into, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ContextObjectId(pub Uuid);

impl ContextObjectId {
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

#[derive(Debug, From, Into, PartialEq, Eq, Clone, Copy, Hash)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

#[derive(Debug, From, Into, PartialEq, Eq, Clone, Copy, Hash)]
pub struct MessageId(pub Uuid);

impl MessageId {
    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}
