diesel::table! {
    sessions (session_id) {
        session_id -> Uuid,
        owner_id -> Uuid,
        current_location_id -> Nullable<Uuid>,
        rng_seed -> Int8,
        rng_counter -> Int8,
        status -> Varchar,
        config -> Jsonb,
        created_ts -> Timestamptz,
        last_activity_ts -> Timestamptz,
        closed_ts -> Nullable<Timestamptz>,
    }
}
