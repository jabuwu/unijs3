use tracing::{info, Level};
use unijs3::{eval, Array, AsObject, Value};

fn main() {
    unilog::init(Level::INFO, "");
    let array = Array::from(vec![
        0.0.into(),
        "hello".into(),
        vec![636.0.into()].into(),
    ]);
    let function = eval("function hi(arg) { console.log(arg) }; hi").into_function().unwrap();
    function.call(vec![array.into()]);
    function.as_object().set("asdf", Value::Number(1.));
    info!("{:?}", function.as_object().get("asdf"));
    let f = Value::from(function.clone());
    function.call(vec![f]);
}
