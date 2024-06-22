use crate::{AsObject, Object, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    array: js_sys::Array,
}

impl Array {
    pub fn new() -> Self {
        Self {
            array: js_sys::Array::new(),
        }
    }

    pub fn length(&self) -> u32 {
        self.array.length()
    }

    pub fn get(&self, index: u32) -> Value {
        Value::from(self.array.get(index))
    }

    pub fn set(&self, index: u32, value: impl Into<Value>) {
        self.array.set(index, wasm_bindgen::JsValue::from(value.into()))
    }

    pub fn push(&self, value: impl Into<Value>) {
        self.array.push(&wasm_bindgen::JsValue::from(value.into()));
    }
}

impl AsObject for Array {
    fn as_object(&self) -> Object {
        Object::from(js_sys::Object::from(self.array.clone()))
    }
}

impl From<Vec<Value>> for Array {
    fn from(vec: Vec<Value>) -> Self {
        let array = Array::new();
        for value in vec {
            array.push(value);
        }
        array
    }
}

#[cfg(target_arch = "wasm32")]
impl From<js_sys::Array> for Array {
    fn from(array: js_sys::Array) -> Self {
        Self { array }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Array> for wasm_bindgen::JsValue {
    fn from(array: Array) -> Self {
        wasm_bindgen::JsValue::from(array.array)
    }
}
