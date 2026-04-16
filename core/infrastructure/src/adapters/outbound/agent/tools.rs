use autoagents::{
    core::tool::ToolCallError,
    prelude::{ToolInput, ToolInputT, ToolRuntime, ToolT},
};
use autoagents_derive::tool;
use serde::Deserialize;
use serde_json::{Value, json};

use super::parsing::{context_object_type_name, normalize_object_type, parse_attributes_value};

#[derive(Debug, Deserialize, ToolInput)]
pub(super) struct CreateContextObjectToolInput {
    #[input(description = "Type of memory object to create", choice = ["Npc", "Place", "Item", "Event", "Note"])]
    object_type: String,
    #[input(description = "Short canonical title of the context object")]
    title: String,
    #[input(description = "One-sentence summary used for retrieval")]
    short_desc: String,
    #[input(description = "Longer optional description with stable details")]
    long_desc: Option<String>,
    #[input(description = "Importance score from 0.0 to 1.0")]
    importance_score: f32,
    #[input(description = "Optional JSON object with only string, number, or boolean values")]
    attributes: Option<Value>,
}

#[tool(
    name = "create_context_object",
    description = "Create a normalized context object when the scene introduces a durable world fact worth storing.",
    input = CreateContextObjectToolInput
)]
pub(super) struct CreateContextObjectTool;

#[async_trait::async_trait]
impl ToolRuntime for CreateContextObjectTool {
    async fn execute(&self, args: Value) -> Result<Value, ToolCallError> {
        let input: CreateContextObjectToolInput = serde_json::from_value(args)?;
        let object_type = normalize_object_type(&input.object_type)
            .ok_or_else(|| ToolCallError::RuntimeError("invalid context object type".into()))?;
        let title = input.title.trim();
        let short_desc = input.short_desc.trim();

        if title.is_empty() || short_desc.is_empty() {
            return Err(ToolCallError::RuntimeError(
                "title and short_desc must be non-empty".into(),
            ));
        }

        let attributes = parse_attributes_value(input.attributes)?;

        Ok(json!({
            "object_type": context_object_type_name(object_type),
            "title": title,
            "short_desc": short_desc,
            "long_desc": input.long_desc.as_deref().map(str::trim).filter(|value| !value.is_empty()),
            "attributes": attributes,
            "importance_score": input.importance_score.clamp(0.0, 1.0),
        }))
    }
}
