diesel::table! {
    use crate::repositories::messages::MessageRoleDbMapping;
    use diesel::sql_types::*;

    messages(id) {
        id -> Uuid,
        session_id -> Uuid,
        role -> MessageRoleDbMapping,
        text -> Text,
        ts -> Timestamptz,
    }
}
