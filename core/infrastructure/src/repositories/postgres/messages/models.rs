use diesel::sql_types::SqlType;
use diesel_derive_enum::DbEnum;

#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
#[PgType = "MessageRole"]
pub enum MessageRoleDb {
    Player,
    Gm,
    System,
}
