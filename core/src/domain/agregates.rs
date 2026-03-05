use std::collections::HashMap;

use anyhow::{Result, bail};
use chrono::{DateTime, Utc};
use serde_json::Value;
use uuid::Uuid;

use crate::domain::value_objects::{
    ContextObjectType, GameSessionConfig, GameSessionMode, Provenance,
};

/// Agregate
#[derive(Debug)]
pub struct GameSession {
    owner_id: Uuid,
    session_mode: GameSessionMode,
    config: GameSessionConfig,
}

/// Agregate
#[derive(Debug)]
pub struct GameSessionMetadata {
    id: Uuid,
    created_ts: DateTime<Utc>,
    last_activity_ts: Option<DateTime<Utc>>,
}

impl GameSession {
    pub fn new(owner_id: Uuid, session_mode: GameSessionMode, config: GameSessionConfig) -> Self {
        Self {
            owner_id,
            session_mode,
            config,
        }
    }

    pub fn restore(
        owner_id: Uuid,
        session_mode: GameSessionMode,
        config: GameSessionConfig,
    ) -> Self {
        Self {
            owner_id,
            session_mode,
            config,
        }
    }

    pub fn owner_id(&self) -> Uuid {
        self.owner_id
    }

    pub fn session_mode(&self) -> &GameSessionMode {
        &self.session_mode
    }

    pub fn config(&self) -> &GameSessionConfig {
        &self.config
    }
}

impl GameSessionMetadata {
    pub fn new(
        id: Uuid,
        created_ts: DateTime<Utc>,
        last_activity_ts: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            created_ts,
            last_activity_ts,
        }
    }

    pub fn restore(
        id: Uuid,
        created_ts: DateTime<Utc>,
        last_activity_ts: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id,
            created_ts,
            last_activity_ts,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn created_ts(&self) -> DateTime<Utc> {
        self.created_ts
    }

    pub fn last_activity_ts(&self) -> Option<DateTime<Utc>> {
        self.last_activity_ts
    }
}

/// Agregate
#[derive(Debug)]
pub struct ContextObject {
    id: Uuid,
    object_type: ContextObjectType,
    title: String,
    short_desc: String,
    long_desc: Option<String>,
    attributes: HashMap<String, Value>,
    place_id: Option<Uuid>,
    importance_score: f32,
}

/// Agregate
#[derive(Debug)]
pub struct ContextObjectMetadata {
    id: Uuid,
    created_ts: DateTime<Utc>,
    updated_ts: Option<DateTime<Utc>>,
    provenance: Provenance,
}

impl ContextObject {
    fn validate_attributes(attributes: &HashMap<String, Value>) -> Result<()> {
        for key in attributes.keys() {
            if key.trim().is_empty() {
                bail!("ContextObject attributes must not contain empty keys");
            }
        }

        Ok(())
    }

    fn validate(
        title: &str,
        short_desc: &str,
        long_desc: Option<&str>,
        importance_score: f32,
    ) -> Result<()> {
        if title.trim().is_empty() {
            bail!("ContextObject title must not be empty");
        }

        if short_desc.trim().is_empty() {
            bail!("ContextObject short_desc must not be empty");
        }

        if long_desc.is_some_and(|d| d.trim().is_empty()) {
            bail!("ContextObject long_desc must not be empty when provided");
        }

        if !importance_score.is_finite() {
            bail!("ContextObject importance_score must be a finite number");
        }

        Ok(())
    }

    pub fn new(
        object_type: ContextObjectType,
        title: &str,
        short_desc: &str,
        long_desc: Option<&str>,
        attributes: HashMap<String, Value>,
        place_id: Option<Uuid>,
        importance_score: f32,
    ) -> Result<Self> {
        Self::validate(title, short_desc, long_desc, importance_score)?;
        Self::validate_attributes(&attributes)?;

        Ok(Self {
            id: Uuid::new_v4(),
            object_type,
            title: title.to_owned(),
            short_desc: short_desc.to_owned(),
            long_desc: long_desc.map(str::to_owned),
            attributes, // TODO: validate?
            place_id,
            importance_score,
        })
    }

    pub fn restore(
        id: Uuid,
        object_type: ContextObjectType,
        title: String,
        short_desc: String,
        long_desc: Option<String>,
        attributes: HashMap<String, Value>,
        place_id: Option<Uuid>,
        importance_score: f32,
    ) -> Result<Self> {
        Self::validate(&title, &short_desc, long_desc.as_deref(), importance_score)?;
        Self::validate_attributes(&attributes)?;

        Ok(Self {
            id,
            object_type,
            title,
            short_desc,
            long_desc,
            attributes,
            place_id,
            importance_score,
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
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

    pub fn attributes(&self) -> &HashMap<String, Value> {
        &self.attributes
    }

    pub fn place_id(&self) -> Option<Uuid> {
        self.place_id
    }

    pub fn importance_score(&self) -> f32 {
        self.importance_score
    }
}

impl ContextObjectMetadata {
    pub fn new(created_ts: DateTime<Utc>, provenance: Provenance) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            created_ts,
            updated_ts: None,
            provenance,
        })
    }

    pub fn restore(
        id: Uuid,
        created_ts: DateTime<Utc>,
        updated_ts: Option<DateTime<Utc>>,
        provenance: Provenance,
    ) -> Result<Self> {
        Ok(Self {
            id,
            created_ts,
            updated_ts,
            provenance,
        })
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn created_ts(&self) -> DateTime<Utc> {
        self.created_ts
    }

    pub fn updated_ts(&self) -> Option<DateTime<Utc>> {
        self.updated_ts
    }

    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
}
