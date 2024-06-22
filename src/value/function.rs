use crate::{AsObject, Object, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    function: js_sys::Function,
}

impl Function {
    // TODO: use an iterator instead of vec
    pub fn call(&self, args: Vec<Value>) -> Value {
        let array = js_sys::Array::new();
        for arg in args {
            array.push(&wasm_bindgen::JsValue::from(arg));
        }
        let ret = self
            .function
            .apply(&wasm_bindgen::JsValue::null(), &array)
            .unwrap();
        Value::from(ret)
    }
}

impl AsObject for Function {
    fn as_object(&self) -> Object {
        Object::from(js_sys::Object::from(self.function.clone()))
    }
}

#[cfg(target_arch = "wasm32")]
impl From<js_sys::Function> for Function {
    fn from(function: js_sys::Function) -> Self {
        Self { function }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Function> for wasm_bindgen::JsValue {
    fn from(function: Function) -> Self {
        wasm_bindgen::JsValue::from(function.function)
    }
}
