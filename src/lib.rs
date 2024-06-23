mod value;

pub use value::*;

pub mod native;
pub mod json;

#[cfg(not(target_arch = "wasm32"))]
mod v8;

// TODO: remove unwraps and return error
pub fn eval(source: impl AsRef<str>) -> Value {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let scope = crate::v8::scope();
        let code = ::v8::String::new(scope, source.as_ref()).unwrap();
        let script = ::v8::Script::compile(scope, code, None).unwrap();
        Value::from(script.run(scope).unwrap())
    }
    #[cfg(target_arch = "wasm32")]
    {
        Value::from(js_sys::eval(source.as_ref()).unwrap())
    }
}

#[cfg(target_arch = "wasm32")]
fn global_object() -> wasm_bindgen::JsValue {
    if let Some(window) = web_sys::window() {
        wasm_bindgen::JsValue::from(window)
    } else if let Value::Object(global) = eval("global") {
        wasm_bindgen::JsValue::from(js_sys::Object::from(global))
    } else {
        panic!()
    }
}

pub fn global_set(name: impl AsRef<str>, value: impl Into<Value>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let scope = crate::v8::scope();
        let context = crate::v8::context();
        let global = context.global(scope);
        let name = ::v8::String::new(scope, name.as_ref()).unwrap();
        let value = ::v8::Local::<::v8::Value>::from(value.into());
        global.set(scope, name.into(), value).unwrap();
    }
    #[cfg(target_arch = "wasm32")]
    {
        let global = global_object();
        js_sys::Reflect::set(
            &global,
            &name.as_ref().into(),
            &wasm_bindgen::JsValue::from(value.into()),
        )
        .unwrap();
    }
}

pub fn global_get(name: impl AsRef<str>) -> Value {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let scope = crate::v8::scope();
        let context = crate::v8::context();
        let global = context.global(scope);
        let name = ::v8::String::new(scope, name.as_ref()).unwrap();
        let value = global.get(scope, name.into());
        if let Some(value) = value {
            Value::from(value)
        } else {
            Value::Undefined
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let global = global_object();
        Value::from(js_sys::Reflect::get(&global, &name.as_ref().into()).unwrap())
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::{global_get, global_set, Value};

    #[test]
    fn global_set_get() {
        global_set("test", Value::Number(636.));
        assert_eq!(global_get("test"), Value::Number(636.));
    }
}