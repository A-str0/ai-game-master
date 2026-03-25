/// ValueObject
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    Text(String),
    Number(f64),
    Bool(bool),
}
