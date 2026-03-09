/// ValueObject
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextObjectType {
    Npc,
    Place,
    Item,
    Event,
    Note,
}
