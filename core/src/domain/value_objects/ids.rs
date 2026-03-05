use derive_more::{From, Into};
use uuid::Uuid;

#[derive(Debug, From, Into, PartialEq, Clone, Copy, Hash)]
pub struct GameSessionId(pub Uuid);

impl GameSessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

#[derive(Debug, From, Into, PartialEq, Clone, Copy, Hash)]
pub struct ContextObjectId(pub Uuid);

impl ContextObjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

#[derive(Debug, From, Into, PartialEq, Clone, Copy, Hash)]
pub struct UserId(pub Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}
