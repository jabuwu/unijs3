use crate::{AsObject, Object, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    #[cfg(not(target_arch = "wasm32"))]
    function: v8::Global<v8::Function>,
    #[cfg(target_arch = "wasm32")]
    function: js_sys::Function,
}

impl Function {
    pub fn call(&self, args: impl IntoIterator<Item = Value>) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let function = v8::Local::new(scope, self.function.clone());
            let recv = v8::null(scope);
            let args = args
                .into_iter()
                .map(|value| v8::Local::<v8::Value>::from(value))
                .collect::<Vec<_>>();
            let ret = function.call(scope, recv.into(), &args).unwrap();
            Value::from(ret)
        }
        #[cfg(target_arch = "wasm32")]
        {
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
}

impl AsObject for Function {
    fn as_object(&self) -> Object {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let local = v8::Local::new(scope, self.function.clone());
            Object::from(v8::Local::<v8::Object>::from(local))
        }
        #[cfg(target_arch = "wasm32")]
        {
            Object::from(js_sys::Object::from(self.function.clone()))
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<v8::Local<'_, v8::Function>> for Function {
    fn from(value: v8::Local<v8::Function>) -> Self {
        let scope = crate::v8::scope();
        Self {
            function: v8::Global::new(scope, value),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Function> for v8::Local<'_, v8::Function> {
    fn from(value: Function) -> Self {
        let scope = crate::v8::scope();
        v8::Local::new(scope, &value.function)
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

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::eval;

    #[test]
    fn call_js_function() {
        let function = eval("function test() { return 'b' + 'a' + + 'a' + 'a'; }; test")
            .into_function()
            .unwrap();
        let result = function.call([]).into_string().unwrap();
        assert_eq!(result, "baNaNa");
    }
}
