use std::fmt::Display;

#[derive(Debug)]
pub enum StateError {
    NotAttribute(String),
    NotObject(String),
}

impl Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateError::NotObject(m) => write!(f, "{} does not indicate an object.", m),
            StateError::NotAttribute(m) => {
                write!(f, "{} does not indicate an attribute tag.", m)
            }
        }
    }
}

#[derive(Debug)]
pub struct AttributeMatchError {
    current: String,
    expect: String,
}

impl Display for AttributeMatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "The current attribute: {} does not match the expected: {}.",
            self.current, self.expect
        )
    }
}
