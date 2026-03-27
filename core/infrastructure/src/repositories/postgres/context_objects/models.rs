use std::collections::HashMap;

use crate::repositories::postgres::connecion::{PgRepositoryError, PgRepositoryResult};
use chrono::{DateTime, Utc};
use diesel::sql_types::SqlType;
use diesel::{AsChangeset, Identifiable, Insertable, Queryable};
use diesel_derive_enum::DbEnum;
use domain::{
    Identifiable as DomainIdentifiable,
    aggregates::ContextObject,
    value_objects::{
        AttributeValue, ContextObjectId, ContextObjectType, GameSessionId, Provenance,
    },
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
pub(crate) struct ContextObjectRow {
    pub id: Uuid,
    pub session_id: Uuid,
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

impl TryFrom<ContextObjectRow> for ContextObject {
    type Error = PgRepositoryError;

    fn try_from(row: ContextObjectRow) -> PgRepositoryResult<Self> {
        ContextObject::restore(
            ContextObjectId(row.id),
            GameSessionId(row.session_id),
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
        .map_err(|error| PgRepositoryError::Internal {
            details: format!("context_object row violates domain invariants: {error}"),
        })
    }
}

#[derive(Debug, Insertable)]
#[diesel(table_name = context_objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewContextObjectRow {
    pub id: Uuid,
    pub session_id: Uuid,
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

impl From<&ContextObject> for NewContextObjectRow {
    fn from(value: &ContextObject) -> Self {
        Self {
            id: value.id().0,
            session_id: value.session_id().0,
            object_type: (*value.object_type()).into(),
            title: value.title().to_owned(),
            short_desc: value.short_desc().to_owned(),
            long_desc: value.long_desc().cloned(),
            attributes: attributes_to_json(value.attributes()),
            place_id: value.place_id().map(|id| id.0),
            importance_score: value.importance_score(),
            created_by: value.provenance().created_by().to_owned(),
            seed: value.provenance().seed(),
            created_ts: value.created_ts(),
            updated_ts: value.updated_ts(),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = context_objects)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct ContextObjectChangeset {
    pub session_id: Uuid,
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

impl From<&ContextObject> for ContextObjectChangeset {
    fn from(value: &ContextObject) -> Self {
        Self {
            session_id: value.session_id().0,
            object_type: (*value.object_type()).into(),
            title: value.title().to_owned(),
            short_desc: value.short_desc().to_owned(),
            long_desc: value.long_desc().cloned(),
            attributes: attributes_to_json(value.attributes()),
            place_id: value.place_id().map(|id| id.0),
            importance_score: value.importance_score(),
            created_by: value.provenance().created_by().to_owned(),
            seed: value.provenance().seed(),
            updated_ts: value.updated_ts(),
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

fn json_to_attributes(value: Value) -> PgRepositoryResult<HashMap<String, AttributeValue>> {
    let Value::Object(object) = value else {
        return Err(PgRepositoryError::Internal {
            details: String::from("context_object attributes column is not a JSON object"),
        });
    };

    object
        .into_iter()
        .map(|(key, value)| json_to_attribute(value).map(|value| (key, value)))
        .collect()
}

fn json_to_attribute(value: Value) -> PgRepositoryResult<AttributeValue> {
    match value {
        Value::String(text) => Ok(AttributeValue::Text(text)),
        Value::Number(number) => {
            number
                .as_f64()
                .map(AttributeValue::Number)
                .ok_or(PgRepositoryError::Internal {
                    details: String::from("numeric attribute cannot be represented as f64"),
                })
        }
        Value::Bool(flag) => Ok(AttributeValue::Bool(flag)),
        _ => Err(PgRepositoryError::Internal {
            details: String::from("attributes may contain only string, number, or bool values"),
        }),
    }
}
