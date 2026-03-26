use domain::aggregates::{ContextObject, GameSession, Message};

use crate::AppResult;

#[derive(Debug, Clone)]
pub struct RetrivialObject {
    pub context_object: ContextObject,
    pub semantic_similarity: f32,
    pub combined_score: f32,
}

#[async_trait::async_trait]
pub trait RetrivialService: Send + Sync {
    async fn find_for_message(
        &self,
        session: &GameSession,
        player_message: &Message,
    ) -> AppResult<Vec<RetrivialObject>>;
}
