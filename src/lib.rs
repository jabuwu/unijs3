mod value;
mod gc;

pub use value::*;
pub use gc::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod v8;

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
        use web_sys::window;
        let window = window().unwrap();
        js_sys::Reflect::set(
            &window.into(),
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
        use web_sys::window;
        let window = window().unwrap();
        Value::from(js_sys::Reflect::get(&window.into(), &name.as_ref().into()).unwrap())
    }
}
