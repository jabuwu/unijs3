mod value;

pub use value::*;

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
