use application::ports::{
    MemoryExtractorRequest, NarratorContextObject, NarratorMessage, NarratorRequest,
};
use domain::value_objects::MessageRole;

pub(super) fn build_narrator_task(request: &NarratorRequest) -> String {
    format!(
        "{system_prompt}

You are the GM for a tabletop RPG.
Write the next response for the player.
If the exchange introduces a durable world fact that should be remembered later, call the tool `create_context_object` exactly once before your final answer.
If the player explicitly asks to remember someone or something, you must call `create_context_object`.
When a named NPC, place, item, event, or long-term note first becomes relevant, prefer creating a context object instead of skipping it.
Only use the tool for persistent NPCs, places, items, events, or notes. Do not use it for transient narration or repeated facts.
When you create an `Npc` context object, include known or directly inferable `race` and `class`/profession in `attributes` when available.
For NPCs, keep `long_desc` omitted unless the player already established specific biography; a separate backstory model will fill it.
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
        system_prompt = request.system_prompt,
        world_summary = request.world_summary,
        recent_messages = format_recent_messages(&request.recent_messages),
        retrieved_objects = format_retrieved_objects(&request.retrieved_objects),
        player_action = request.player_action,
        instructions = request.instructions,
    )
}

pub(super) fn build_extraction_task(request: &MemoryExtractorRequest) -> String {
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
        recent_messages = format_recent_messages(&request.recent_messages),
        player_action = request.player_action,
        narrator_message = request.narrator_message,
        retrieved_objects = format_retrieved_objects(&request.retrieved_objects),
    )
}

fn format_recent_messages(messages: &[NarratorMessage]) -> String {
    if messages.is_empty() {
        return String::from("- none");
    }

    messages
        .iter()
        .map(|message| format!("- {}: {}", message_role_name(message.role), message.text))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_retrieved_objects(objects: &[NarratorContextObject]) -> String {
    if objects.is_empty() {
        return String::from("- none");
    }

    objects
        .iter()
        .map(|object| format!("- {}: {}", object.title, object.summary))
        .collect::<Vec<_>>()
        .join("\n")
}

fn message_role_name(value: MessageRole) -> &'static str {
    match value {
        MessageRole::Player => "player",
        MessageRole::Gm => "gm",
        MessageRole::System => "system",
    }
}
