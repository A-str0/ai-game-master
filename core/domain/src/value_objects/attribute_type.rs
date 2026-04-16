/// Primitive attribute value stored inside a context object.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    /// Free-form textual attribute.
    Text(String),
    /// Numeric attribute value.
    Number(f64),
    /// Boolean attribute value.
    Bool(bool),
}
