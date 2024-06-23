use crate::Value;

pub fn stringify(value: impl Into<Value>) -> Option<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let scope = crate::v8::scope();
        let value = v8::Local::<v8::Value>::from(value.into());
        let result = v8::json::stringify(scope, value)?;
        Some(result.to_rust_string_lossy(scope))
    }
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::JSON::stringify(&wasm_bindgen::JsValue::from(value.into())).ok()?.as_string()
    }
}

pub fn parse(string: impl AsRef<str>) -> Option<Value> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let scope = crate::v8::scope();
        let string = v8::String::new(scope, string.as_ref())?;
        Some(Value::from(v8::json::parse(scope, string.into())?))
    }
    #[cfg(target_arch = "wasm32")]
    {
        Some(Value::from(js_sys::JSON::parse(string.as_ref()).ok()?))
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::{json, Object, Value};

    #[test]
    fn stringify() {
        let object = Object::new();
        object.set("foo", "bar");
        let string = json::stringify(object);
        assert_eq!(string, Some("{\"foo\":\"bar\"}".to_owned()));
    }

    #[test]
    fn parse() {
        let object = json::parse("{\"foo\":\"bar\"}")
            .unwrap()
            .into_object()
            .unwrap();
        assert_eq!(object.get("foo"), Value::from("bar"));
    }
}
