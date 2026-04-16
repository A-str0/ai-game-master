/// High-level play mode for a session.
#[derive(Debug, Clone, Copy)]
pub enum GameSessionMode {
    /// The session is controlled by a single player.
    Solo,
    /// The session is shared across multiple players.
    Multi,
}
