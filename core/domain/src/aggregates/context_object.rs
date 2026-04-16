use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{
    DomainError, DomainResult, Identifiable,
    value_objects::{
        AttributeValue, ContextObjectId, ContextObjectType, GameSessionId, Provenance,
    },
};

/// Aggregate root that stores durable world state for a session.
///
/// A context object can describe an NPC, place, item, event, or free-form note
/// that should remain retrievable across turns.
#[derive(Debug, Clone)]
pub struct ContextObject {
    id: ContextObjectId,
    session_id: GameSessionId,
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

    /// Creates a new context object after validating its content.
    pub fn new(
        id: ContextObjectId,
        session_id: GameSessionId,
        object_type: ContextObjectType,
        title: &str,
        short_desc: &str,
        long_desc: Option<&str>,
        attributes: HashMap<String, AttributeValue>,
        place_id: Option<ContextObjectId>,
        importance_score: f32,
        provenance: Provenance,
        created_ts: DateTime<Utc>,
    ) -> DomainResult<Self> {
        Self::validate(title, short_desc, long_desc, importance_score)?;
        Self::validate_attributes(&attributes)?;

        Ok(Self {
            id,
            session_id,
            object_type,
            title: title.to_owned(),
            short_desc: short_desc.to_owned(),
            long_desc: long_desc.map(str::to_owned),
            attributes,
            place_id,
            importance_score,
            provenance,
            created_ts,
            updated_ts: None,
        })
    }

    /// Restores a context object from already persisted state.
    pub fn restore(
        id: ContextObjectId,
        session_id: GameSessionId,
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
            session_id,
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

    /// Returns the domain category of the stored memory.
    pub fn object_type(&self) -> &ContextObjectType {
        &self.object_type
    }

    /// Returns the human-readable name used to refer to this object.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the compact summary used during retrieval and prompting.
    pub fn short_desc(&self) -> &str {
        &self.short_desc
    }

    /// Returns the extended description, when one was captured.
    pub fn long_desc(&self) -> Option<&String> {
        self.long_desc.as_ref()
    }

    /// Returns free-form structured attributes extracted for the object.
    pub fn attributes(&self) -> &HashMap<String, AttributeValue> {
        &self.attributes
    }

    /// Returns the containing place when this object is anchored to one.
    pub fn place_id(&self) -> Option<&ContextObjectId> {
        self.place_id.as_ref()
    }

    /// Returns the importance score later consumed by retrieval scoring.
    pub fn importance_score(&self) -> f32 {
        self.importance_score
    }

    /// Returns provenance metadata describing where the object came from.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// Returns when the object was first created.
    pub fn created_ts(&self) -> DateTime<Utc> {
        self.created_ts
    }

    /// Returns the latest activity timestamp, if the object was updated later.
    pub fn updated_ts(&self) -> Option<DateTime<Utc>> {
        self.updated_ts
    }

    /// Returns the session that owns this context object.
    pub fn session_id(&self) -> GameSessionId {
        self.session_id
    }

    /// Updates the activity timestamp used by recency-based retrieval.
    pub fn mark_activity(&mut self, ts: DateTime<Utc>) {
        self.updated_ts = Some(ts)
    }
}

impl Identifiable for ContextObject {
    type Id = ContextObjectId;

    fn id(&self) -> &Self::Id {
        &self.id
    }
}
