use std::collections::HashMap;

use application::ports::{RepoError, RepoResult};
use chrono::{DateTime, Utc};
use diesel::sql_types::SqlType;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable};
use diesel_derive_enum::DbEnum;
use domain::{
    Identifiable as DomainIdentifiable,
    aggregates::ContextObject,
    value_objects::{AttributeValue, ContextObjectId, ContextObjectType, Provenance},
};
use serde_json::{Map, Number, Value};
use uuid::Uuid;

use super::schema::context_objects;

#[derive(Debug, Clone, Copy, DbEnum, SqlType)]
#[PgType = "ContextObjectType"]
#[DbValueStyle = "PascalCase"]
pub enum ContextObjectTypeDb {
    Npc,
    Place,
    Item,
    Event,
    Note,
}

impl From<ContextObjectType> for ContextObjectTypeDb {
    fn from(value: ContextObjectType) -> Self {
        match value {
            ContextObjectType::Npc => Self::Npc,
            ContextObjectType::Place => Self::Place,
            ContextObjectType::Item => Self::Item,
            ContextObjectType::Event => Self::Event,
            ContextObjectType::Note => Self::Note,
        }
    }
}

impl From<ContextObjectTypeDb> for ContextObjectType {
    fn from(value: ContextObjectTypeDb) -> Self {
        match value {
            ContextObjectTypeDb::Npc => Self::Npc,
            ContextObjectTypeDb::Place => Self::Place,
            ContextObjectTypeDb::Item => Self::Item,
            ContextObjectTypeDb::Event => Self::Event,
            ContextObjectTypeDb::Note => Self::Note,
        }
    }
}

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = context_objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContextObjectRow {
    pub id: Uuid,
    pub object_type: ContextObjectTypeDb,
    pub title: String,
    pub short_desc: String,
    pub long_desc: Option<String>,
    pub attributes: Value,
    pub place_id: Option<Uuid>,
    pub importance_score: f32,
    pub created_by: String,
    pub seed: i64,
    pub created_ts: DateTime<Utc>,
    pub updated_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = context_objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewContextObjectRow {
    pub id: Uuid,
    pub object_type: ContextObjectTypeDb,
    pub title: String,
    pub short_desc: String,
    pub long_desc: Option<String>,
    pub attributes: Value,
    pub place_id: Option<Uuid>,
    pub importance_score: f32,
    pub created_by: String,
    pub seed: i64,
    pub created_ts: DateTime<Utc>,
    pub updated_ts: Option<DateTime<Utc>>,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = context_objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ContextObjectChangeset {
    pub object_type: ContextObjectTypeDb,
    pub title: String,
    pub short_desc: String,
    pub long_desc: Option<String>,
    pub attributes: Value,
    pub place_id: Option<Uuid>,
    pub importance_score: f32,
    pub created_by: String,
    pub seed: i64,
    pub updated_ts: Option<DateTime<Utc>>,
}

impl TryFrom<ContextObjectRow> for ContextObject {
    type Error = RepoError;

    fn try_from(row: ContextObjectRow) -> RepoResult<Self> {
        ContextObject::restore(
            ContextObjectId(row.id),
            row.object_type.into(),
            row.title,
            row.short_desc,
            row.long_desc,
            json_to_attributes(row.attributes)?,
            row.place_id.map(ContextObjectId),
            row.importance_score,
            Provenance::restore(row.created_by, row.seed),
            row.created_ts,
            row.updated_ts,
        )
        .map_err(|_| RepoError::Unavailable)
    }
}

impl From<&ContextObject> for NewContextObjectRow {
    fn from(object: &ContextObject) -> Self {
        Self {
            id: object.id().0,
            object_type: (*object.object_type()).into(),
            title: object.title().to_owned(),
            short_desc: object.short_desc().to_owned(),
            long_desc: object.long_desc().cloned(),
            attributes: attributes_to_json(object.attributes()),
            place_id: object.place_id().map(|id| id.0),
            importance_score: object.importance_score(),
            created_by: object.provenance().created_by().to_owned(),
            seed: object.provenance().seed(),
            created_ts: object.created_ts(),
            updated_ts: object.updated_ts(),
        }
    }
}

impl From<&ContextObject> for ContextObjectChangeset {
    fn from(object: &ContextObject) -> Self {
        Self {
            object_type: (*object.object_type()).into(),
            title: object.title().to_owned(),
            short_desc: object.short_desc().to_owned(),
            long_desc: object.long_desc().cloned(),
            attributes: attributes_to_json(object.attributes()),
            place_id: object.place_id().map(|id| id.0),
            importance_score: object.importance_score(),
            created_by: object.provenance().created_by().to_owned(),
            seed: object.provenance().seed(),
            updated_ts: object.updated_ts(),
        }
    }
}

fn attributes_to_json(attributes: &HashMap<String, AttributeValue>) -> Value {
    let mut object = Map::with_capacity(attributes.len());

    for (key, value) in attributes {
        object.insert(key.clone(), attribute_to_json(value));
    }

    Value::Object(object)
}

fn attribute_to_json(value: &AttributeValue) -> Value {
    match value {
        AttributeValue::Text(text) => Value::String(text.clone()),
        AttributeValue::Number(number) => Number::from_f64(*number)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        AttributeValue::Bool(flag) => Value::Bool(*flag),
    }
}

fn json_to_attributes(value: Value) -> RepoResult<HashMap<String, AttributeValue>> {
    let Value::Object(object) = value else {
        return Err(RepoError::Unavailable);
    };

    object
        .into_iter()
        .map(|(key, value)| json_to_attribute(value).map(|value| (key, value)))
        .collect()
}

fn json_to_attribute(value: Value) -> RepoResult<AttributeValue> {
    match value {
        Value::String(text) => Ok(AttributeValue::Text(text)),
        Value::Number(number) => number
            .as_f64()
            .map(AttributeValue::Number)
            .ok_or(RepoError::Unavailable),
        Value::Bool(flag) => Ok(AttributeValue::Bool(flag)),
        _ => Err(RepoError::Unavailable),
    }
}
