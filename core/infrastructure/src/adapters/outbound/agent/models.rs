use application::ports::ProposedContextObject;
use autoagents::prelude::{AgentOutputT, ReActAgentOutput};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::parsing::parse_context_object;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct NarratorOutput {
    pub(super) message: String,
    #[serde(skip, default)]
    pub(super) context_object: Option<ProposedContextObject>,
    #[serde(skip, default)]
    pub(super) tool_call_attempted: bool,
    #[serde(skip, default)]
    pub(super) tool_call_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct MemoryExtractorOutput {
    pub(super) create_context_object: bool,
    pub(super) object_type: Option<String>,
    pub(super) title: Option<String>,
    pub(super) short_desc: Option<String>,
    pub(super) long_desc: Option<String>,
    pub(super) importance_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NarratorMessagePayload {
    message: String,
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
