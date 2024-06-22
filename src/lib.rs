mod value;

pub use value::*;

pub fn eval(source: &str) -> Value {
    Value::from(js_sys::eval(source).unwrap())
}
