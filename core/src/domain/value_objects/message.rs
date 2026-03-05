/// ValueObject
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    Player,
    Gm,
    System,
}
