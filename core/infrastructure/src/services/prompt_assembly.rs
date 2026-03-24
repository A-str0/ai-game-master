use application::{AppResult, ports::AgentPrompt, services::PromptAssemblyService};
use domain::aggregates::{GameSession, Message};

struct PromptAssembly;

// TODO: move to config
const SYSTEM_PROMPT: &str = "You are the Game Master for a tabletop fantasy role‐playing game. Be vivid, coherent and consistent with earlier world details. Use retrieved context objects below when relevant. If a player asks about a new entity or requests interaction, either act using existing objects or return an explicit action to \"CREATE_OBJECT\" with a short spec. Keep answers in-character and use dice notations when needed (e.g., \"roll 1d20+3\").";
const INSTRUCTIONS: &str = "- Use the retrieved objects to answer; if you need more detail for an object that is only implied, output: CREATE_OBJECT: <brief spec>. \n- If player action involves a contested roll, include \"ROLL: <dice_expr>\" and a short explanation of consequences. \n- Output must be JSON at the end with keys: \"response\": \"<text>\", \"actions\":";

#[allow(unused_variables)] // TODO: remove
#[async_trait::async_trait]
impl PromptAssemblyService for PromptAssembly {
    async fn assemble(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> AppResult<AgentPrompt> {
        Ok(AgentPrompt {
            system_prompt: String::from(SYSTEM_PROMPT),
            world_summary: String::from("TODO"), // TODO
            retrieved_objects: Vec::new(),       // TODO
            player_action: String::from(player_message.text()), // TODO
            instructions: String::from(INSTRUCTIONS),
        })
    }
}
