use crate::Value;

pub trait AsObject {
    fn as_object(&self) -> Object;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    object: js_sys::Object,
}

impl Object {
    pub fn new() -> Self {
        Self {
            object: js_sys::Object::new(),
        }
    }

    pub fn get(&self, key: &str) -> Value {
        let key = wasm_bindgen::JsValue::from_str(key);
        js_sys::Reflect::get(&self.object, &key)
            .map(|value| Value::from(value))
            .unwrap_or_else(|_| Value::Undefined)
    }

    pub fn set(&self, key: &str, value: Value) {
        let key = wasm_bindgen::JsValue::from_str(key);
        // TODO: don't unwrap
        js_sys::Reflect::set(&self.object, &key, &wasm_bindgen::JsValue::from(value)).unwrap();
    }
}

impl AsObject for Object {
    fn as_object(&self) -> Object {
        self.clone()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<js_sys::Object> for Object {
    fn from(object: js_sys::Object) -> Self {
        Self { object }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Object> for wasm_bindgen::JsValue {
    fn from(object: Object) -> Self {
        wasm_bindgen::JsValue::from(object.object)
    }
}
