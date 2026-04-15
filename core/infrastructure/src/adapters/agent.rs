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
    #[serde(skip, default)]
    tool_call_attempted: bool,
    #[serde(skip, default)]
    tool_call_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryExtractorOutput {
    create_context_object: bool,
    object_type: Option<String>,
    title: Option<String>,
    short_desc: Option<String>,
    long_desc: Option<String>,
    importance_score: Option<f32>,
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

        let create_context_call = output
            .tool_calls
            .iter()
            .rev()
            .find(|call| call.tool_name == "create_context_object");

        let (tool_call_attempted, context_object, tool_call_error) = match create_context_call {
            None => (false, None, None),
            Some(call) if !call.success => (
                true,
                None,
                Some(format!(
                    "agent attempted create_context_object but the tool call failed: {}",
                    call.result
                )),
            ),
            Some(call) => match parse_context_object(&call.result) {
                Ok(object) => (true, Some(object), None),
                Err(error) => (
                    true,
                    None,
                    Some(format!(
                        "agent returned malformed create_context_object payload ({error}): {}",
                        call.result
                    )),
                ),
            },
        };

        Self {
            message,
            context_object,
            tool_call_attempted,
            tool_call_error,
        }
    }
}

impl AgentOutputT for MemoryExtractorOutput {
    fn output_schema() -> &'static str {
        r#"{
            "name":"MemoryExtractorOutput",
            "description":"Decision about whether the latest exchange should create a durable context object",
            "schema":{
                "type":"object",
                "properties":{
                    "create_context_object":{"type":"boolean","description":"Whether a new durable context object should be created from the exchange."},
                    "object_type":{"type":["string","null"],"enum":["Npc","Place","Item","Event","Note",null]},
                    "title":{"type":["string","null"]},
                    "short_desc":{"type":["string","null"]},
                    "long_desc":{"type":["string","null"]},
                    "importance_score":{"type":["number","null"]}
                },
                "required":["create_context_object","object_type","title","short_desc","long_desc","importance_score"],
                "additionalProperties":false
            },
            "strict":true
        }"#
    }

    fn structured_output_format() -> Value {
        serde_json::from_str(Self::output_schema())
            .expect("MemoryExtractorOutput schema must be valid")
    }
}

impl From<ReActAgentOutput> for MemoryExtractorOutput {
    fn from(output: ReActAgentOutput) -> Self {
        serde_json::from_str(&output.response).unwrap_or(Self {
            create_context_object: false,
            object_type: None,
            title: None,
            short_desc: None,
            long_desc: None,
            importance_score: None,
        })
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
    #[input(description = "Optional JSON object with only string, number, or boolean values")]
    attributes: Option<Value>,
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

#[derive(Clone, Copy, AgentHooks, Default)]
#[agent(
    name = "Narrator",
    description = "Narrates scenes and uses tools to create durable context objects when needed.",
    tools = [CreateContextObjectTool],
    output = NarratorOutput
)]
pub struct Narrator;

#[derive(Clone, Copy, AgentHooks, Default)]
#[agent(
    name = "MemoryExtractor",
    description = "Extracts at most one durable context object from the latest exchange.",
    output = MemoryExtractorOutput
)]
pub struct MemoryExtractor;

pub struct DefaultAgentOrchestrator {
    narrator_handle: DirectAgentHandle<ReActAgent<Narrator>>,
    extractor_handle: DirectAgentHandle<ReActAgent<MemoryExtractor>>,
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

        let narrator = ReActAgent::new(Narrator);
        let narrator_handle = AgentBuilder::<_, DirectAgent>::new(narrator)
            .llm(llm)
            .memory(Box::new(SlidingWindowMemory::new(10)))
            .build()
            .await?;

        let api_key = std::env::var("LLM_API_KEY").unwrap_or_else(|_| String::from("API_KEY"));
        let model = std::env::var("LLM_MODEL")
            .unwrap_or_else(|_| String::from("nvidia/nemotron-3-super-120b-a12b:free"));
        let extractor_llm = LLMBuilder::<OpenRouter>::new()
            .api_key(api_key)
            .model(model)
            .build()?;
        let extractor = ReActAgent::new(MemoryExtractor);
        let extractor_handle = AgentBuilder::<_, DirectAgent>::new(extractor)
            .llm(extractor_llm)
            .memory(Box::new(SlidingWindowMemory::new(10)))
            .build()
            .await?;

        Ok(Self {
            narrator_handle,
            extractor_handle,
        })
    }
}

#[async_trait::async_trait]
impl AgentOrchestrator for DefaultAgentOrchestrator {
    async fn generate(&self, prompt: &PromptInput) -> AgentResult<AgentOrchestratorResponse> {
        let output = self
            .narrator_handle
            .agent
            .run(Task::new(build_task(prompt)))
            .await
            .map_err(|error| AgentOrchestratorError::Unavailable {
                details: format!("agent execution failed: {error}"),
            })?;

        if let Some(details) = output.tool_call_error {
            return Err(AgentOrchestratorError::InvalidResponse { details });
        }

        if let Some(object) = output.context_object {
            return Ok(AgentOrchestratorResponse::CreateContextObjects {
                message: output.message,
                objects: vec![object],
            });
        }

        if !output.tool_call_attempted {
            eprintln!(
                "narrator returned text without create_context_object tool call; player_action={:?}",
                prompt.player_action
            );
        }

        if output.message.trim().is_empty() {
            return Err(AgentOrchestratorError::InvalidResponse {
                details: String::from("agent returned an empty final message"),
            });
        }

        if let Some(object) = self.extract_context_object(prompt, &output.message).await? {
            return Ok(AgentOrchestratorResponse::CreateContextObjects {
                message: output.message,
                objects: vec![object],
            });
        }

        Ok(AgentOrchestratorResponse::Text(output.message))
    }
}

impl DefaultAgentOrchestrator {
    async fn extract_context_object(
        &self,
        prompt: &PromptInput,
        narrator_message: &str,
    ) -> AgentResult<Option<ProposedContextObject>> {
        let output = self
            .extractor_handle
            .agent
            .run(Task::new(build_extraction_task(prompt, narrator_message)))
            .await
            .map_err(|error| AgentOrchestratorError::Unavailable {
                details: format!("memory extraction failed: {error}"),
            })?;

        extracted_object_into_proposed(output)
    }
}

fn build_task(prompt: &PromptInput) -> String {
    format!(
        "{system_prompt}

You are the GM for a tabletop RPG.
Write the next response for the player.
If the exchange introduces a durable world fact that should be remembered later, call the tool `create_context_object` exactly once before your final answer.
If the player explicitly asks to remember someone or something, you must call `create_context_object`.
When a named NPC, place, item, event, or long-term note first becomes relevant, prefer creating a context object instead of skipping it.
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

fn build_extraction_task(prompt: &PromptInput, narrator_message: &str) -> String {
    format!(
        "You decide whether the latest RPG exchange should create a durable context object.
Create one only when the exchange introduces or confirms a persistent NPC, place, item, event, or note that will matter later.
If the player explicitly asks to remember someone or something, prefer create_context_object = true.
Extract at most one object. If nothing durable was introduced, return create_context_object = false and null for all other fields.
Do not invent facts beyond the conversation.

Recent conversation:
{recent_messages}

Player action:
{player_action}

Narrator response:
{narrator_message}

Retrieved objects:
{retrieved_objects}",
        recent_messages = format_recent_messages(&prompt.recent_messages),
        player_action = prompt.player_action,
        narrator_message = narrator_message,
        retrieved_objects = format_retrieved_objects(&prompt.retrieved_objects),
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

fn parse_attributes_value(value: Option<Value>) -> Result<Value, ToolCallError> {
    let Some(value) = value else {
        return Ok(Value::Object(Default::default()));
    };

    let Value::Object(_) = value else {
        return Err(ToolCallError::RuntimeError(
            "attributes must be a JSON object".into(),
        ));
    };

    parse_attribute_values(value.clone())
        .map(|_| value)
        .map_err(|error| ToolCallError::RuntimeError(error.into()))
}

fn extracted_object_into_proposed(
    output: MemoryExtractorOutput,
) -> AgentResult<Option<ProposedContextObject>> {
    if !output.create_context_object {
        return Ok(None);
    }

    let object_type = output
        .object_type
        .as_deref()
        .and_then(normalize_object_type)
        .ok_or_else(|| AgentOrchestratorError::InvalidResponse {
            details: String::from(
                "memory extractor decided to create a context object but returned an invalid object_type",
            ),
        })?;
    let title = output
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AgentOrchestratorError::InvalidResponse {
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
        .ok_or_else(|| AgentOrchestratorError::InvalidResponse {
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
        .ok_or_else(|| AgentOrchestratorError::InvalidResponse {
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
