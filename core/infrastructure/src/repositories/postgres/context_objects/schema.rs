diesel::table! {
    use crate::repositories::context_objects::ContextObjectTypeDbMapping;
    use diesel::sql_types::*;

    context_objects(id) {
        id -> Uuid,
        object_type -> ContextObjectTypeDbMapping,
        title -> Text,
        short_desc -> Text,
        long_desc -> Nullable<Text>,
        attributes -> Json,
        place_id -> Nullable<Uuid>,
        importance_score -> Float4,
        created_by -> Text,
        seed -> BigInt,
        created_ts -> Timestamptz,
        updated_ts -> Nullable<Timestamptz>,
    }
}
