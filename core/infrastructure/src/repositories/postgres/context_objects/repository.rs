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
    DieselResultExt, FromPgRepositoryError, PgPool, PgRepositoryError, PgRepositoryResultExt,
    RESOURCE_CONTEXT_OBJECT, connection,
};

/// Postgres implementation of [`ContextObjectRepository`].
#[derive(Clone)]
pub struct PgContextObjectRepository {
    pool: PgPool,
}

impl PgContextObjectRepository {
    /// Creates a repository backed by the supplied Diesel pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl FromPgRepositoryError for ContextObjectRepositoryError {
    fn from_pg_repository_error(error: PgRepositoryError) -> Self {
        match error {
            PgRepositoryError::NotFound { details, .. } => Self::NotFound { details },
            PgRepositoryError::Conflict { details, .. } => Self::Conflict { details },
            PgRepositoryError::Unavailable { details } => Self::Unavailable { details },
            PgRepositoryError::Internal { details } => Self::Internal { details },
        }
    }
}

#[async_trait::async_trait]
impl ContextObjectRepository for PgContextObjectRepository {
    async fn insert(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()> {
        let mut conn = connection(&self.pool).into_repo()?;
        let row = NewContextObjectRow::from(context_object);

        diesel::insert_into(dsl::context_objects)
            .values(&row)
            .execute(&mut conn)
            .into_repo_diesel(RESOURCE_CONTEXT_OBJECT)?;

        Ok(())
    }

    async fn get_by_id(
        &self,
        session_id: &GameSessionId,
        id: &ContextObjectId,
    ) -> ContextObjectRepositoryResult<ContextObject> {
        let mut conn = connection(&self.pool).into_repo()?;
        let row = dsl::context_objects
            .filter(dsl::session_id.eq(session_id.0))
            .filter(dsl::id.eq(id.0))
            .first::<ContextObjectRow>(&mut conn)
            .into_repo_diesel(RESOURCE_CONTEXT_OBJECT)?;

        row.try_into().into_repo()
    }

    async fn update(&self, context_object: &ContextObject) -> ContextObjectRepositoryResult<()> {
        let mut conn = connection(&self.pool).into_repo()?;
        let changes = ContextObjectChangeset::from(context_object);

        let updated_rows = diesel::update(
            dsl::context_objects
                .filter(dsl::session_id.eq(context_object.session_id().0))
                .filter(dsl::id.eq(context_object.id().0)),
        )
        .set(&changes)
        .execute(&mut conn)
        .into_repo_diesel(RESOURCE_CONTEXT_OBJECT)?;

        if updated_rows == 0 {
            return Err(ContextObjectRepositoryError::NotFound {
                details: format!(
                    "no rows updated for context_object {} in session {}",
                    context_object.id().0,
                    context_object.session_id().0
                ),
            });
        }

        Ok(())
    }
}
