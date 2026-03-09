use crate::{Identifiable, value_objects::UserId};

/// Aggregate
#[derive(Debug)]
pub struct User {
    id: UserId,
}

impl Identifiable for User {
    type Id = UserId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}

impl User {
    pub fn new(id: UserId) -> Self {
        Self { id }
    }
}
