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

    pub fn as_boolean(&self) -> Option<&bool> {
        if let Self::Boolean(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_number(&self) -> Option<&f64> {
        if let Self::Number(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<&String> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_array(&self) -> Option<&Array> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&Object> {
        if let Self::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<&Function> {
        if let Self::Function(value) = self {
            Some(value)
        } else {
            None
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: nicer output for array, object
        match self {
            Self::Undefined => write!(f, "undefined"),
            Self::Null => write!(f, "null"),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::Number(value) => write!(f, "{}", value),
            Self::String(value) => write!(f, "\"{}\"", &value),
            Self::Array(value) => write!(f, "{}", value),
            Self::Object(value) => write!(f, "{}", value),
            Self::Function(value) => write!(f, "{}", value),
        }
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Undefined
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

#[cfg(not(target_arch = "wasm32"))]
impl From<v8::Local<'_, v8::Value>> for Value {
    fn from(value: v8::Local<v8::Value>) -> Self {
        let scope = crate::v8::scope();
        if value.is_undefined() {
            Self::Undefined
        } else if value.is_null() {
            Self::Null
        } else if value.is_boolean() {
            Self::Boolean(value.boolean_value(scope))
        } else if value.is_number() {
            Self::Number(value.number_value(scope).unwrap())
        } else if value.is_string() {
            // TODO: this impl kinda sucks?
            let string: v8::Local<v8::String> = value.try_into().unwrap();
            let mut buffer = [0; 1024];
            let mut nchars = 0;
            string.write_utf8(
                scope,
                &mut buffer,
                Some(&mut nchars),
                v8::WriteOptions::default(),
            );
            let string = std::str::from_utf8(&buffer).unwrap().to_owned();
            Self::String(string.chars().take(nchars).collect())
        } else if value.is_function() {
            Self::Function(Function::from(
                v8::Local::<v8::Function>::try_from(value).unwrap(),
            ))
        } else if value.is_array() {
            // TODO: remove unwrap?
            Self::Array(Array::from(
                v8::Local::<v8::Array>::try_from(value).unwrap(),
            ))
        } else if value.is_object() {
            // TODO: remove unwrap?
            Self::Object(Object::from(
                v8::Local::<v8::Object>::try_from(value).unwrap(),
            ))
        } else {
            todo!("{:?}", value.to_rust_string_lossy(scope))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Value> for v8::Local<'_, v8::Value> {
    fn from(value: Value) -> Self {
        let scope = crate::v8::scope();
        match value {
            Value::Undefined => v8::undefined(scope).into(),
            Value::Null => v8::null(scope).into(),
            Value::Boolean(value) => v8::Boolean::new(scope, value).into(),
            Value::Number(value) => v8::Number::new(scope, value).into(),
            Value::String(value) => v8::String::new(scope, value.as_str()).unwrap().into(),
            Value::Array(value) => {
                v8::Local::<v8::Value>::from(v8::Local::<v8::Array>::from(value))
            }
            Value::Object(value) => {
                v8::Local::<v8::Value>::from(v8::Local::<v8::Object>::from(value))
            }
            Value::Function(value) => {
                v8::Local::<v8::Value>::from(v8::Local::<v8::Function>::from(value))
            }
        }
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
            Value::Array(value) => {
                let array: js_sys::Array = From::<Array>::from(value);
                let array: wasm_bindgen::JsValue = array.into();
                array
            }
            Value::Object(value) => wasm_bindgen::JsValue::from(js_sys::Object::from(value)),
            Value::Function(value) => wasm_bindgen::JsValue::from(js_sys::Function::from(value)),
        }
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::{eval, Array, Function, Object, Value};

    #[test]
    fn debug() {
        assert_eq!(format!("{:?}", Value::Undefined), "Undefined");
        assert_eq!(format!("{:?}", Value::Null), "Null");
        assert_eq!(format!("{:?}", Value::Boolean(true)), "Boolean(true)");
        assert_eq!(format!("{:?}", Value::Boolean(false)), "Boolean(false)");
        assert_eq!(format!("{:?}", Value::Number(3.)), "Number(3.0)");
        assert_eq!(format!("{:?}", Value::Number(3.3)), "Number(3.3)");
        assert_eq!(
            format!("{:?}", Value::String("hello".to_owned())),
            "String(\"hello\")"
        );
        assert_eq!(format!("{:?}", Value::Array(Array::new())), "Array([])");
        assert_eq!(format!("{:?}", Value::Object(Object::new())), "Object({})");
        assert_eq!(
            format!("{:?}", Value::Function(Function::new(|_| {}))),
            "Function(ƒ())"
        );
    }

    #[test]
    fn display() {
        assert_eq!(Value::Undefined.to_string(), "undefined");
        assert_eq!(Value::Null.to_string(), "null");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Boolean(false).to_string(), "false");
        assert_eq!(Value::Number(3.).to_string(), "3");
        assert_eq!(Value::Number(3.3).to_string(), "3.3");
        assert_eq!(Value::String("hello".to_owned()).to_string(), "\"hello\"");

        assert_eq!(Value::Array(Array::new()).to_string(), "[]");
        assert_eq!(
            Value::Array(Array::from(vec![
                0.0.into(),
                "hello".into(),
                Object::new().into(),
                Function::new(|_| {}).into()
            ]))
            .to_string(),
            "[0, \"hello\", {}, ƒ()]"
        );

        assert_eq!(Value::Object(Object::new()).to_string(), "{}");
        let object = Object::new();
        object.set("foo", "bar");
        object.set("pi", 3.14);
        object.set("obj", Object::new());
        object.set("arr", Array::new());
        object.set("fn", Function::new(|_| {}));
        assert_eq!(
            object.to_string(),
            "{ foo: \"bar\", pi: 3.14, obj: {}, arr: [], fn: ƒ() }"
        );

        assert_eq!(Value::Function(Function::new(|_| {})).to_string(), "ƒ()");
        assert_eq!(eval("function hi() {}; hi").to_string(), "hi()");
    }

    #[test]
    fn convert_from() {
        assert_eq!(eval("let a = {}; a.b"), Value::Undefined);
        assert_eq!(eval("null"), Value::Null);
        assert_eq!(eval("true"), Value::Boolean(true));
        assert_eq!(eval("false"), Value::Boolean(false));
        assert_eq!(eval("1 + 2"), Value::Number(3.));
        assert_eq!(eval("'hello'"), Value::String("hello".to_owned()));

        let array = eval("[1, 2, 3]").into_array().unwrap();
        assert_eq!(array.length(), 3);
        // TODO: test Array, Object, Function
    }

    #[test]
    fn convert_into() {
        fn check_equal(value: Value, js: &str) {
            let check = eval(format!(
                "function check(value) {{ return value === {}; }}; check",
                js
            ))
            .into_function()
            .unwrap();
            if !check.call([value.clone()]).into_boolean().unwrap() {
                panic!("Conversion failed: {} != {}", value, js);
            }
        }
        check_equal(Value::Undefined, "undefined");
        check_equal(Value::Null, "null");
        check_equal(Value::Boolean(true), "true");
        check_equal(Value::Boolean(false), "false");
        check_equal(Value::Number(3.), "3");
        check_equal(Value::Number(3.3), "3.3");
        check_equal(Value::String("hello".to_owned()), "\"hello\"");
        // TODO: Array, Object, Function
    }
}
