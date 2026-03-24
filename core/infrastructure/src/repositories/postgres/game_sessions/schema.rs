diesel::table! {
    use crate::repositories::game_sessions::SessionModeDbMapping;
    use diesel::sql_types::*;

    game_sessions(id) {
        id -> Uuid,
        owner_id -> Uuid,
        retrivial_k -> Int2,
        memory_budget -> Integer,
        session_mode -> SessionModeDbMapping,
        created_ts -> Timestamptz,
        last_activity_ts -> Nullable<Timestamptz>,
    }
}
