/// Role of a message inside the session transcript.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    /// Message authored by the player.
    Player,
    /// Message authored by the game master.
    Gm,
    /// Message authored by internal system logic.
    System,
}
