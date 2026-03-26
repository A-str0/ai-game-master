use diesel::sql_types::SqlType;
use diesel_derive_enum::DbEnum;

#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
#[PgType = "ContextObject"]
#[DbValueStyle = "PascalCase"]
pub enum ContextObjectTypeDb {
    Solo,
    Multi,
}
