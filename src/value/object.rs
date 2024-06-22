use crate::Value;

pub trait AsObject {
    fn as_object(&self) -> Object;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    #[cfg(not(target_arch = "wasm32"))]
    object: v8::Global<v8::Object>,
    #[cfg(target_arch = "wasm32")]
    object: js_sys::Object,
}

impl Object {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Object::new(scope);
            Self {
                object: v8::Global::new(scope, object),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                object: js_sys::Object::new(),
            }
        }
    }

    pub fn get(&self, key: &str) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            let name = v8::String::new(scope, key).unwrap();
            let value = object.get(scope, name.into());
            if let Some(value) = value {
                Value::from(value)
            } else {
                Value::Undefined
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let key = wasm_bindgen::JsValue::from_str(key);
            js_sys::Reflect::get(&self.object, &key)
                .map(|value| Value::from(value))
                .unwrap_or_else(|_| Value::Undefined)
        }
    }

    pub fn set(&self, key: &str, value: impl Into<Value>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            let name = v8::String::new(scope, key).unwrap();
            let value = v8::Local::<v8::Value>::from(value.into());
            object.set(scope, name.into(), value).unwrap();
        }
        #[cfg(target_arch = "wasm32")]
        {
            let key = wasm_bindgen::JsValue::from_str(key);
            // TODO: don't unwrap
            js_sys::Reflect::set(&self.object, &key, &wasm_bindgen::JsValue::from(value.into())).unwrap();
        }
    }
}

impl AsObject for Object {
    fn as_object(&self) -> Object {
        self.clone()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<v8::Local<'_, v8::Object>> for Object {
    fn from(value: v8::Local<v8::Object>) -> Self {
        let scope = crate::v8::scope();
        Self {
            object: v8::Global::new(scope, value),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Object> for v8::Local<'_, v8::Object> {
    fn from(value: Object) -> Self {
        let scope = crate::v8::scope();
        v8::Local::new(scope, &value.object)
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

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::Object;

    #[test]
    fn set_get() {
        let object = Object::new();
        object.set("foo", "bar");
        assert_eq!(object.get("foo").into_string().unwrap(), "bar");
    }
}
