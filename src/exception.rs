use crate::Value;

pub struct Exception(Value);

impl Exception {
    pub fn msg(message: impl AsRef<str>) -> Self {
        // TODO: instantiate Error type
        Self(Value::String(message.as_ref().to_owned()))
    }
}

impl From<&str> for Exception {
    fn from(value: &str) -> Self {
        Exception::msg(value)
    }
}

impl From<String> for Exception {
    fn from(value: String) -> Self {
        Exception::msg(value)
    }
}

impl From<Value> for Exception {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl From<Exception> for Value {
    fn from(value: Exception) -> Self {
        value.0
    }
}

pub trait OrThrow where Self: Sized {
    type Value;

    fn or_throw(self, exception: impl Into<Exception>) -> Result<Self::Value, Exception>;
}

impl<T> OrThrow for Option<T> {
    type Value = T;
    fn or_throw(self, exception: impl Into<Exception>) -> Result<Self::Value, Exception> {
        match self {
            Some(ok) => Ok(ok),
            None => Err(exception.into()),
        }
    }
}

impl<T, E> OrThrow for Result<T, E> {
    type Value = T;
    fn or_throw(self, exception: impl Into<Exception>) -> Result<Self::Value, Exception> {
        match self {
            Ok(ok) => Ok(ok),
            Err(_) => Err(exception.into()),
        }
    }
}
