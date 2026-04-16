/// Semantic category assigned to a stored context object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextObjectType {
    /// A character or creature.
    Npc,
    /// A location, room, settlement, or region.
    Place,
    /// A portable object or piece of equipment.
    Item,
    /// A noteworthy happening in the world.
    Event,
    /// Free-form durable information that does not fit another category.
    Note,
}
