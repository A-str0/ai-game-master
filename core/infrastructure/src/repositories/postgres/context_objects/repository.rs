use application::ports::{
    ContextObjectRepository, ContextObjectRepositoryError, ContextObjectRepositoryResult,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use domain::{
    Identifiable,
    aggregates::ContextObject,
    value_objects::{ContextObjectId, GameSessionId},
};

use super::{
    models::{ContextObjectChangeset, ContextObjectRow, NewContextObjectRow},
    schema::context_objects::dsl,
};
use crate::repositories::postgres::connecion::{
    PgPool, PgRepositoryError, RESOURCE_CONTEXT_OBJECT, connection, map_diesel_error,
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
    async fn create(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()> {
        let mut conn = connection(&self.pool).map_err(map_repository_error)?;
        let row = NewContextObjectRow::from(context_object);

        diesel::insert_into(dsl::context_objects)
            .values(&row)
            .execute(&mut conn)
            .map_err(|error| {
                map_repository_error(map_diesel_error(error, RESOURCE_CONTEXT_OBJECT))
            })?;

        Ok(())
    }

    async fn get_by_id(
        &self,
        session_id: &GameSessionId,
        id: &ContextObjectId,
    ) -> ContextObjectRepositoryResult<ContextObject> {
        let mut conn = connection(&self.pool).map_err(map_repository_error)?;
        let row = dsl::context_objects
            .filter(dsl::session_id.eq(session_id.0))
            .filter(dsl::id.eq(id.0))
            .first::<ContextObjectRow>(&mut conn)
            .map_err(|error| {
                map_repository_error(map_diesel_error(error, RESOURCE_CONTEXT_OBJECT))
            })?;

        row.try_into().map_err(map_repository_error)
    }

    async fn update(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()> {
        let mut conn = connection(&self.pool).map_err(map_repository_error)?;
        let changes = ContextObjectChangeset::from(context_object);

        let updated_rows = diesel::update(
            dsl::context_objects
                .filter(dsl::session_id.eq(context_object.session_id().0))
                .filter(dsl::id.eq(context_object.id().0)),
        )
        .set(&changes)
        .execute(&mut conn)
        .map_err(|error| map_repository_error(map_diesel_error(error, RESOURCE_CONTEXT_OBJECT)))?;

        if updated_rows == 0 {
            return Err(ContextObjectRepositoryError::not_found(
                RESOURCE_CONTEXT_OBJECT,
                format!(
                    "no rows updated for context object {} in session {}",
                    context_object.id().0,
                    context_object.session_id().0
                ),
            ));
        }

        Ok(())
    }
}

fn map_repository_error(error: PgRepositoryError) -> ContextObjectRepositoryError {
    match error {
        PgRepositoryError::NotFound { resource, details } => {
            ContextObjectRepositoryError::NotFound { resource, details }
        }
        PgRepositoryError::Conflict { resource, details } => {
            ContextObjectRepositoryError::Conflict { resource, details }
        }
        PgRepositoryError::Unavailable { details } => {
            ContextObjectRepositoryError::Unavailable { details }
        }
        PgRepositoryError::Internal { details } => {
            ContextObjectRepositoryError::Internal { details }
        }
    }
}
