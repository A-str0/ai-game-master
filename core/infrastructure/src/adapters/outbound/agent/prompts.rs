use application::ports::{
    BackstoryGenerationRequest, MemoryExtractorRequest, NarratorContextObject, NarratorMessage,
    NarratorRequest,
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
When you create an `Npc` context object and the scene establishes who that character is, use `long_desc` for a concise backstory or biography with stable details that may matter later.
Prefer putting biography, allegiances, origin, motives, reputation, and defining past events into `long_desc` rather than bloating `short_desc`.
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

pub(super) fn build_backstory_task(request: &BackstoryGenerationRequest) -> String {
    format!(
        "Generate a concise NPC backstory for a context object that the narrator has already decided to persist.
Return only the final backstory text for the context object's long_desc field.
Do not add headings, bullet lists, JSON, or meta commentary.
Preserve all established facts. You may infer restrained connective details only when they fit the turn context.
Focus on stable details useful for future retrieval: origin, reputation, allegiances, motives, defining past events, secrets, and current stakes.

World summary:
{world_summary}

Recent conversation:
{recent_messages}

Retrieved objects:
{retrieved_objects}

Player action:
{player_action}

Narrator response:
{narrator_message}

NPC context object:
Title: {title}
Short description: {short_desc}
Existing long description: {long_desc}",
        world_summary = request.world_summary,
        recent_messages = format_recent_messages(&request.recent_messages),
        retrieved_objects = format_retrieved_objects(&request.retrieved_objects),
        player_action = request.player_action,
        narrator_message = request.narrator_message,
        title = request.context_object.title,
        short_desc = request.context_object.short_desc,
        long_desc = request
            .context_object
            .long_desc
            .as_deref()
            .unwrap_or("not supplied"),
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
