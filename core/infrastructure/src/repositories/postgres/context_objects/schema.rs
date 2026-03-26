diesel::table! {
    use crate::repositories::context_objects::ContextObjectTypeDbMapping;
    use diesel::sql_types::*;

    context_objects(id) {
        id -> Uuid,
        object_type -> ContextObjectTypeDbMapping,
        title -> VarChar,
        short_desc -> VarChar,
        long_desc -> Nullable<VarChar>,
        attributes -> Json,
        place_id -> Nullable<Uuid>,
        importance_score -> Integer,
        created_by -> VarChar,
        seed -> BigInt,
        created_ts -> Timestamptz,
        updated_ts -> Nullable<Timestamptz>,
    }
}
