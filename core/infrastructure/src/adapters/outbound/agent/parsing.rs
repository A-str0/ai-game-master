use std::collections::HashMap;

use application::ports::{MemoryExtractorError, MemoryExtractorResult, ProposedContextObject};
use domain::value_objects::{AttributeValue, ContextObjectType};
use serde_json::Value;

use crate::adapters::outbound::agent::models::MemoryExtractorOutput;

pub(super) fn parse_context_object(value: &Value) -> Result<ProposedContextObject, &'static str> {
    let object_type = value
        .get("object_type")
        .and_then(Value::as_str)
        .and_then(normalize_object_type)
        .ok_or("invalid object_type")?;
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or("missing title")?
        .to_owned();
    let short_desc = value
        .get("short_desc")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or("missing short_desc")?
        .to_owned();
    let long_desc = value
        .get("long_desc")
        .and_then(|value| match value {
            Value::Null => Some(None),
            Value::String(text) => Some(Some(text.trim().to_owned())),
            _ => None,
        })
        .ok_or("invalid long_desc")?
        .filter(|value| !value.is_empty());
    let attributes = parse_attribute_values(
        value
            .get("attributes")
            .cloned()
            .unwrap_or_else(|| Value::Object(Default::default())),
    )?;
    let importance_score = value
        .get("importance_score")
        .and_then(Value::as_f64)
        .map(|value| value.clamp(0.0, 1.0) as f32)
        .ok_or("invalid importance_score")?;

    Ok(ProposedContextObject {
        object_type,
        title,
        short_desc,
        long_desc,
        attributes,
        importance_score,
    })
}

pub(super) fn parse_attributes_value(
    value: Option<Value>,
) -> Result<Value, autoagents::core::tool::ToolCallError> {
    let Some(value) = value else {
        return Ok(Value::Object(Default::default()));
    };

    let Value::Object(_) = value else {
        return Err(autoagents::core::tool::ToolCallError::RuntimeError(
            "attributes must be a JSON object".into(),
        ));
    };

    parse_attribute_values(value.clone())
        .map(|_| value)
        .map_err(|error| autoagents::core::tool::ToolCallError::RuntimeError(error.into()))
}

pub(super) fn extracted_object_into_proposed(
    output: MemoryExtractorOutput,
) -> MemoryExtractorResult<Option<ProposedContextObject>> {
    if !output.create_context_object {
        return Ok(None);
    }

    let object_type = output
        .object_type
        .as_deref()
        .and_then(normalize_object_type)
        .ok_or_else(|| MemoryExtractorError::InvalidResponse {
            details: String::from(
                "memory extractor decided to create a context object but returned an invalid object_type",
            ),
        })?;
    let title = output
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| MemoryExtractorError::InvalidResponse {
            details: String::from(
                "memory extractor decided to create a context object but returned an empty title",
            ),
        })?
        .to_owned();
    let short_desc = output
        .short_desc
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| MemoryExtractorError::InvalidResponse {
            details: String::from(
                "memory extractor decided to create a context object but returned an empty short_desc",
            ),
        })?
        .to_owned();
    let long_desc = output
        .long_desc
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    let importance_score = output
        .importance_score
        .map(|value| value.clamp(0.0, 1.0))
        .ok_or_else(|| MemoryExtractorError::InvalidResponse {
            details: String::from(
                "memory extractor decided to create a context object but did not return importance_score",
            ),
        })?;

    Ok(Some(ProposedContextObject {
        object_type,
        title,
        short_desc,
        long_desc,
        attributes: HashMap::new(),
        importance_score,
    }))
}

pub(super) fn normalize_object_type(value: &str) -> Option<ContextObjectType> {
    match value {
        "Npc" => Some(ContextObjectType::Npc),
        "Place" => Some(ContextObjectType::Place),
        "Item" => Some(ContextObjectType::Item),
        "Event" => Some(ContextObjectType::Event),
        "Note" => Some(ContextObjectType::Note),
        _ => None,
    }
}

pub(super) fn context_object_type_name(value: ContextObjectType) -> &'static str {
    match value {
        ContextObjectType::Npc => "Npc",
        ContextObjectType::Place => "Place",
        ContextObjectType::Item => "Item",
        ContextObjectType::Event => "Event",
        ContextObjectType::Note => "Note",
    }
}

fn parse_attribute_values(value: Value) -> Result<HashMap<String, AttributeValue>, &'static str> {
    let Value::Object(attributes) = value else {
        return Err("attributes must be an object");
    };

    attributes
        .into_iter()
        .map(|(key, value)| {
            let attribute = match value {
                Value::String(text) => AttributeValue::Text(text),
                Value::Number(number) => number
                    .as_f64()
                    .map(AttributeValue::Number)
                    .ok_or("invalid number attribute")?,
                Value::Bool(flag) => AttributeValue::Bool(flag),
                _ => return Err("attributes must contain only scalar values"),
            };

            Ok((key, attribute))
        })
        .collect()
}
