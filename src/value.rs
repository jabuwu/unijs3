mod array;
mod function;
mod object;

pub use array::*;
pub use function::*;
pub use object::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Array),
    Object(Object),
    Function(Function),
}

impl Value {
    pub fn is_undefined(self) -> bool {
        matches!(self, Self::Undefined)
    }

    pub fn is_null(self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_boolean(self) -> bool {
        matches!(self, Self::Boolean(..))
    }

    pub fn is_number(self) -> bool {
        matches!(self, Self::Number(..))
    }

    pub fn is_string(self) -> bool {
        matches!(self, Self::String(..))
    }

    pub fn is_array(self) -> bool {
        matches!(self, Self::Array(..))
    }

    pub fn is_object(self) -> bool {
        matches!(self, Self::Object(..))
    }

    pub fn is_function(self) -> bool {
        matches!(self, Self::Function(..))
    }

    pub fn into_boolean(self) -> Option<bool> {
        if let Self::Boolean(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_number(self) -> Option<f64> {
        if let Self::Number(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_string(self) -> Option<String> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_array(self) -> Option<Array> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_object(self) -> Option<Object> {
        if let Self::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn into_function(self) -> Option<Function> {
        if let Self::Function(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_owned())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<Vec<Value>> for Value {
    fn from(value: Vec<Value>) -> Self {
        Value::Array(Array::from(value))
    }
}

impl From<Array> for Value {
    fn from(value: Array) -> Self {
        Value::Array(value)
    }
}

impl From<Object> for Value {
    fn from(value: Object) -> Self {
        Value::Object(value)
    }
}

impl From<Function> for Value {
    fn from(value: Function) -> Self {
        Value::Function(value)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<wasm_bindgen::JsValue> for Value {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        if value.is_undefined() {
            Value::Undefined
        } else if value.is_null() {
            Value::Null
        } else if let Some(value) = value.as_bool() {
            Self::Boolean(value)
        } else if let Some(value) = value.as_f64() {
            Self::Number(value)
        } else if let Some(value) = value.as_string() {
            Self::String(value)
        } else if value.is_function() {
            Self::Function(Function::from(js_sys::Function::from(value)))
        } else if value.is_array() {
            Self::Array(Array::from(js_sys::Array::from(&value)))
        } else if value.is_object() {
            Self::Object(Object::from(js_sys::Object::from(value)))
        } else {
            // TODO: return undefined?
            unreachable!()
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Value> for wasm_bindgen::JsValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Undefined => wasm_bindgen::JsValue::undefined(),
            Value::Null => wasm_bindgen::JsValue::null(),
            Value::Boolean(value) => wasm_bindgen::JsValue::from_bool(value),
            Value::Number(value) => wasm_bindgen::JsValue::from_f64(value),
            Value::String(value) => wasm_bindgen::JsValue::from_str(&value),
            Value::Array(value) => wasm_bindgen::JsValue::from(value),
            Value::Object(value) => wasm_bindgen::JsValue::from(value),
            Value::Function(value) => wasm_bindgen::JsValue::from(value),
        }
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::{eval, Value};

    #[test]
    fn convert_from() {
        assert_eq!(eval("let a = {}; a.b"), Value::Undefined);
        assert_eq!(eval("null"), Value::Null);
        assert_eq!(eval("true"), Value::Boolean(true));
        assert_eq!(eval("false"), Value::Boolean(false));
        assert_eq!(eval("1 + 2"), Value::Number(3.));
        assert_eq!(eval("'hello'"), Value::String("hello".to_owned()));
        // TODO: test Array, Object, Function
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn convert_into_web() {
        use wasm_bindgen::JsValue;
        assert_eq!(JsValue::from(Value::Undefined), JsValue::undefined());
        assert_eq!(JsValue::from(Value::Null), JsValue::null());
        assert_eq!(
            JsValue::from(Value::Boolean(true)),
            JsValue::from_bool(true)
        );
        assert_eq!(
            JsValue::from(Value::Boolean(false)),
            JsValue::from_bool(false)
        );
        assert_eq!(JsValue::from(Value::Number(3.)), JsValue::from_f64(3.));
        assert_eq!(
            JsValue::from(Value::String("hello".to_owned())),
            JsValue::from_str("hello")
        );
        // TODO: test Array, Object, Function
    }
}
