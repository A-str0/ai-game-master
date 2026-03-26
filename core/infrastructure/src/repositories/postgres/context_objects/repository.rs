use application::ports::{ContextObjectRepository, RepoError, RepoResult};
use diesel::{QueryDsl, RunQueryDsl};
use domain::{Identifiable, aggregates::ContextObject, value_objects::ContextObjectId};

use super::{
    models::{ContextObjectChangeset, ContextObjectRow, NewContextObjectRow},
    schema::context_objects::dsl,
};
use crate::repositories::postgres::connecion::{
    PgPool, RESOURCE_CONTEXT_OBJECT, connection, map_diesel_error,
};

#[derive(Clone)]
pub struct PgContextObjectRepository {
    pool: PgPool,
}

impl PgContextObjectRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ContextObjectRepository for PgContextObjectRepository {
    async fn create(&self, context_object: &ContextObject) -> RepoResult<()> {
        let mut conn = connection(&self.pool)?;
        let row = NewContextObjectRow::from(context_object);

        diesel::insert_into(dsl::context_objects)
            .values(&row)
            .execute(&mut conn)
            .map_err(|error| map_diesel_error(error, RESOURCE_CONTEXT_OBJECT))?;

        Ok(())
    }

    async fn get_by_id(&self, id: &ContextObjectId) -> RepoResult<ContextObject> {
        let mut conn = connection(&self.pool)?;
        let row = dsl::context_objects
            .find(id.0)
            .first::<ContextObjectRow>(&mut conn)
            .map_err(|error| map_diesel_error(error, RESOURCE_CONTEXT_OBJECT))?;

        row.try_into()
    }

    async fn update(&self, context_object: &ContextObject) -> RepoResult<()> {
        let mut conn = connection(&self.pool)?;
        let changes = ContextObjectChangeset::from(context_object);
        let updated_rows = diesel::update(dsl::context_objects.find(context_object.id().0))
            .set(&changes)
            .execute(&mut conn)
            .map_err(|error| map_diesel_error(error, RESOURCE_CONTEXT_OBJECT))?;

        if updated_rows == 0 {
            return Err(RepoError::not_found(RESOURCE_CONTEXT_OBJECT));
        }

        Ok(())
    }
}
