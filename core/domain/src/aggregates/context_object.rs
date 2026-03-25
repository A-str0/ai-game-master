use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{
    DomainError, DomainResult, Identifiable,
    value_objects::{ContextObjectId, ContextObjectType, Provenance},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    Text(String),
    Number(f64),
    Bool(bool),
}

/// Aggregate Root
#[derive(Debug, Clone)]
pub struct ContextObject {
    id: ContextObjectId,
    object_type: ContextObjectType,
    title: String,
    short_desc: String,
    long_desc: Option<String>,
    attributes: HashMap<String, AttributeValue>,
    place_id: Option<ContextObjectId>,
    importance_score: f32,
    provenance: Provenance,
    created_ts: DateTime<Utc>,
    updated_ts: Option<DateTime<Utc>>,
}

impl ContextObject {
    fn validate_attributes(attributes: &HashMap<String, AttributeValue>) -> DomainResult<()> {
        for key in attributes.keys() {
            if key.trim().is_empty() {
                return Err(DomainError::InvariantViolation(String::from(
                    "ContextObject attributes must not contain empty keys",
                )));
            }
        }

        Ok(())
    }

    fn validate(
        title: &str,
        short_desc: &str,
        long_desc: Option<&str>,
        importance_score: f32,
    ) -> DomainResult<()> {
        if title.trim().is_empty() {
            return Err(DomainError::InvariantViolation(String::from(
                "ContextObject title must not be empty",
            )));
        }

        if short_desc.trim().is_empty() {
            return Err(DomainError::InvariantViolation(String::from(
                "ContextObject short_desc must not be empty",
            )));
        }

        if long_desc.is_some_and(|d| d.trim().is_empty()) {
            return Err(DomainError::InvariantViolation(String::from(
                "ContextObject long_desc must not be empty when provided",
            )));
        }

        if !importance_score.is_finite() {
            return Err(DomainError::InvariantViolation(String::from(
                "ContextObject importance_score must be a finite number",
            )));
        }

        Ok(())
    }

    pub fn new(
        id: ContextObjectId,
        object_type: ContextObjectType,
        title: &str,
        short_desc: &str,
        long_desc: Option<&str>,
        attributes: HashMap<String, AttributeValue>,
        place_id: Option<ContextObjectId>,
        importance_score: f32,
        provenance: Provenance,
        created_ts: DateTime<Utc>,
        updated_ts: Option<DateTime<Utc>>,
    ) -> DomainResult<Self> {
        Self::validate(title, short_desc, long_desc, importance_score)?;
        Self::validate_attributes(&attributes)?;

        Ok(Self {
            id,
            object_type,
            title: title.to_owned(),
            short_desc: short_desc.to_owned(),
            long_desc: long_desc.map(str::to_owned),
            attributes,
            place_id,
            importance_score,
            provenance,
            created_ts,
            updated_ts,
        })
    }

    pub fn restore(
        id: ContextObjectId,
        object_type: ContextObjectType,
        title: String,
        short_desc: String,
        long_desc: Option<String>,
        attributes: HashMap<String, AttributeValue>,
        place_id: Option<ContextObjectId>,
        importance_score: f32,
        provenance: Provenance,
        created_ts: DateTime<Utc>,
        updated_ts: Option<DateTime<Utc>>,
    ) -> DomainResult<Self> {
        Ok(Self {
            id,
            object_type,
            title,
            short_desc,
            long_desc,
            attributes,
            place_id,
            importance_score,
            provenance,
            created_ts,
            updated_ts,
        })
    }

    pub fn object_type(&self) -> &ContextObjectType {
        &self.object_type
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn short_desc(&self) -> &str {
        &self.short_desc
    }

    pub fn long_desc(&self) -> Option<&String> {
        self.long_desc.as_ref()
    }

    pub fn attributes(&self) -> &HashMap<String, AttributeValue> {
        &self.attributes
    }

    pub fn place_id(&self) -> Option<&ContextObjectId> {
        self.place_id.as_ref()
    }

    pub fn importance_score(&self) -> f32 {
        self.importance_score
    }
}

impl Identifiable for ContextObject {
    type Id = ContextObjectId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
