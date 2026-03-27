use std::collections::HashMap;

use application::ports::{
    AgentOrchestrator, AgentOrchestratorError, AgentOrchestratorResponse, AgentResult,
    PromptContextObject, PromptInput, PromptMessage, ProposedContextObject,
};
use autoagents::{
    core::{agent::DirectAgentHandle, tool::ToolCallError},
    llm::backends::openrouter::OpenRouter,
    prelude::*,
};
use autoagents_derive::{AgentHooks, agent};
use domain::value_objects::{AttributeValue, ContextObjectType};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarratorOutput {
    message: String,
    #[serde(skip, default)]
    context_object: Option<ProposedContextObject>,
}

impl AgentOutputT for NarratorOutput {
    fn output_schema() -> &'static str {
        r#"{
            "name":"NarratorOutput",
            "description":"Final narrator response",
            "schema":{
                "type":"object",
                "properties":{
                    "message":{"type":"string","description":"The final GM response for the player."}
                },
                "required":["message"]
            },
            "strict":true
        }"#
    }

    fn structured_output_format() -> Value {
        serde_json::from_str(Self::output_schema()).expect("NarratorOutput schema must be valid")
    }
}

impl From<ReActAgentOutput> for NarratorOutput {
    fn from(output: ReActAgentOutput) -> Self {
        let message = serde_json::from_str::<NarratorMessagePayload>(&output.response)
            .map(|payload| payload.message)
            .unwrap_or(output.response);

        let context_object = output
            .tool_calls
            .iter()
            .rev()
            .find(|call| call.success && call.tool_name == "create_context_object")
            .and_then(|call| parse_context_object(&call.result).ok());

        Self {
            message,
            context_object,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NarratorMessagePayload {
    message: String,
}

#[derive(Debug, Deserialize, ToolInput)]
struct CreateContextObjectToolInput {
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
    #[input(
        description = "JSON object string with scalar attributes, for example {\"faction\":\"guild\",\"hostile\":false}"
    )]
    attributes_json: Option<String>,
}

#[tool(
    name = "create_context_object",
    description = "Create a normalized context object when the scene introduces a durable world fact worth storing.",
    input = CreateContextObjectToolInput
)]
pub struct CreateContextObjectTool {}

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

        let attributes = parse_attributes_json(input.attributes_json.as_deref())?;

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

#[derive(Clone, Copy, AgentHooks, Default)]
#[agent(
    name = "Narrator",
    description = "Narrates scenes and uses tools to create durable context objects when needed.",
    tools = [CreateContextObjectTool],
    output = NarratorOutput
)]
pub struct Narrator;

pub struct DefaultAgentOrchestrator {
    handle: DirectAgentHandle<ReActAgent<Narrator>>,
}

impl DefaultAgentOrchestrator {
    pub async fn new() -> Result<Self, Error> {
        let api_key = std::env::var("LLM_API_KEY").unwrap_or_else(|_| String::from("API_KEY"));
        let model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| String::from("nvidia/nemotron-3-super-120b-a12b:free"));

        let llm = LLMBuilder::<OpenRouter>::new()
            .api_key(api_key)
            .model(model)
            .build()?;

        let agent = ReActAgent::new(Narrator);
        let handle = AgentBuilder::<_, DirectAgent>::new(agent)
            .llm(llm)
            .memory(Box::new(SlidingWindowMemory::new(10)))
            .build()
            .await?;

        Ok(Self { handle })
    }
}

#[async_trait::async_trait]
impl AgentOrchestrator for DefaultAgentOrchestrator {
    async fn generate(&self, prompt: &PromptInput) -> AgentResult<AgentOrchestratorResponse> {
        let output = self
            .handle
            .agent
            .run(Task::new(build_task(prompt)))
            .await
            .map_err(|_| AgentOrchestratorError::Unavailable)?;

        if let Some(object) = output.context_object {
            return Ok(AgentOrchestratorResponse::CreateContextObject {
                message: output.message,
                object,
            });
        }

        Ok(AgentOrchestratorResponse::Text(output.message))
    }
}

fn build_task(prompt: &PromptInput) -> String {
    format!(
        "{system_prompt}

You are the GM for a tabletop RPG.
Write the next response for the player.
If the exchange introduces a durable world fact that should be remembered later, call the tool `create_context_object` exactly once before your final answer.
Only use the tool for persistent NPCs, places, items, events, or notes. Do not use it for transient narration or repeated facts.
Your final answer must contain only the player-facing GM response in the structured `message` field.

World summary:
{world_summary}

Recent conversation:
{recent_messages}

Retrieved objects:
{retrieved_objects}

Player action:
{player_action}

Instructions:
{instructions}",
        system_prompt = prompt.system_prompt,
        world_summary = prompt.world_summary,
        recent_messages = format_recent_messages(&prompt.recent_messages),
        retrieved_objects = format_retrieved_objects(&prompt.retrieved_objects),
        player_action = prompt.player_action,
        instructions = prompt.instructions,
    )
}

fn format_recent_messages(messages: &[PromptMessage]) -> String {
    if messages.is_empty() {
        return String::from("- none");
    }

    messages
        .iter()
        .map(|message| format!("- {}: {}", message.role, message.text))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_retrieved_objects(objects: &[PromptContextObject]) -> String {
    if objects.is_empty() {
        return String::from("- none");
    }

    objects
        .iter()
        .map(|object| format!("- {}: {}", object.title, object.summary))
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_context_object(value: &Value) -> Result<ProposedContextObject, &'static str> {
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

fn parse_attributes_json(raw: Option<&str>) -> Result<Value, ToolCallError> {
    let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(Value::Object(Default::default()));
    };

    let value: Value = serde_json::from_str(raw)?;
    let Value::Object(_) = value else {
        return Err(ToolCallError::RuntimeError(
            "attributes_json must be a JSON object".into(),
        ));
    };

    parse_attribute_values(value.clone())
        .map(|_| value)
        .map_err(|error| ToolCallError::RuntimeError(error.into()))
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

fn normalize_object_type(value: &str) -> Option<ContextObjectType> {
    match value {
        "Npc" => Some(ContextObjectType::Npc),
        "Place" => Some(ContextObjectType::Place),
        "Item" => Some(ContextObjectType::Item),
        "Event" => Some(ContextObjectType::Event),
        "Note" => Some(ContextObjectType::Note),
        _ => None,
    }
}

fn context_object_type_name(value: ContextObjectType) -> &'static str {
    match value {
        ContextObjectType::Npc => "Npc",
        ContextObjectType::Place => "Place",
        ContextObjectType::Item => "Item",
        ContextObjectType::Event => "Event",
        ContextObjectType::Note => "Note",
    }
}
