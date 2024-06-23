use crate::{AsObject, Object, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    #[cfg(not(target_arch = "wasm32"))]
    function: v8::Global<v8::Function>,
    #[cfg(target_arch = "wasm32")]
    function: js_sys::Function,
}

impl Function {
    pub fn new(body: fn(Args) -> Value) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let body_ptr = v8::Number::new(scope, body as usize as f64);
            let function = v8::Function::builder(
                |v8_scope: &mut v8::HandleScope<'_>,
                 v8_args: v8::FunctionCallbackArguments<'_>,
                 mut v8_ret: v8::ReturnValue<'_>| {
                    crate::v8::push_scope(v8_scope);
                    let this = Value::from(Object::from(v8_args.this()));
                    let mut args = Args { this, args: vec![] };
                    let f: fn(Args) -> Value = unsafe {
                        std::mem::transmute(v8_args.data().number_value(v8_scope).unwrap() as usize)
                    };
                    for i in 0..v8_args.length() {
                        args.args.push(Value::from(v8_args.get(i)));
                    }
                    let value = v8::Local::<v8::Value>::from(f(args));
                    crate::v8::pop_scope();
                    v8_ret.set(value);
                },
            )
            .data(body_ptr.into())
            .build(scope)
            .unwrap();
            Self {
                function: v8::Global::new(scope, function),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::{closure::Closure, JsValue};
            let bindgen_closure =
                Closure::<dyn Fn(JsValue, JsValue) -> JsValue>::new(move |js_this: JsValue, js_args: JsValue| {
                    let this = Value::from(js_this);
                    let mut args = Args { this, args: vec![] };
                    let js_args_array: js_sys::Array = js_args.into();
                    for i in 0..js_args_array.length() {
                        args.args.push(Value::from(js_args_array.get(i)));
                    }
                    let ret = body(args);
                    JsValue::from(ret)
                });
            let closure = Value::from(JsValue::from(bindgen_closure.as_ref()));
            bindgen_closure.forget(); // TODO: don't forget
            let js_wrapper = r#"function wrapper() {
                return wrapper.__fn.apply(null, [this, Array.from(arguments)]);
            }"#;
            let function = crate::eval(&format!("{}; wrapper", js_wrapper)).into_function().unwrap();
            function.as_object().set("__fn", closure);
            function
        }
    }

    pub fn call(&self, args: impl IntoIterator<Item = Value>) -> Value {
        self.call_with(Value::Undefined, args)
    }

    // TODO: add receiver
    pub fn call_with(&self, receiver: Value, args: impl IntoIterator<Item = Value>) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let function = v8::Local::new(scope, self.function.clone());
            let receiver = v8::Local::<v8::Value>::from(receiver);
            let args = args
                .into_iter()
                .map(|value| v8::Local::<v8::Value>::from(value))
                .collect::<Vec<_>>();
            // TODO: don't unwrap
            let ret = function.call(scope, receiver, &args).unwrap();
            Value::from(ret)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let receiver = wasm_bindgen::JsValue::from(receiver);
            let array = js_sys::Array::new();
            for arg in args {
                array.push(&wasm_bindgen::JsValue::from(arg));
            }
            // TODO: don't unwrap
            let ret = self
                .function
                .apply(&receiver, &array)
                .unwrap();
            Value::from(ret)
        }
    }

    pub fn new_instance(&self, args: impl IntoIterator<Item = Value>) -> Object {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let function = v8::Local::new(scope, self.function.clone());
            let args = args
                .into_iter()
                .map(|value| v8::Local::<v8::Value>::from(value))
                .collect::<Vec<_>>();
            // TODO: don't unwrap
            let ret = function.new_instance(scope,  &args).unwrap();
            Object::from(ret)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let array = js_sys::Array::new();
            for arg in args {
                array.push(&wasm_bindgen::JsValue::from(arg));
            }
            // TODO: don't unwrap
            let object = js_sys::Object::from(js_sys::Reflect::construct(&self.function, &array).unwrap());
            Object::from(object)
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

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[function]")
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

#[derive(Debug, Clone)]
pub struct Args {
    this: Value,
    args: Vec<Value>,
}

impl Args {
    pub fn this(&self) -> Value {
        self.this.clone()
    }

    pub fn get(&self, index: u32) -> Value {
        self.args
            .get(index as usize)
            .cloned()
            .unwrap_or_else(|| Value::Undefined)
    }

    pub fn length(&self) -> u32 {
        self.args.len() as u32
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::{eval, Function, Value};

    #[test]
    fn call_native_function() {
        let function = Function::new(|args| {
            let a = args.get(0).into_number().unwrap();
            let b = args.get(1).into_number().unwrap();
            Value::from(a + b)
        });
        let result = function
            .call([6.0.into(), 7.0.into()])
            .into_number()
            .unwrap();
        assert_eq!(result, 13.);
    }

    #[test]
    fn call_js_function() {
        let function = eval("function test(a, b) { return a + b; }; test")
            .into_function()
            .unwrap();
        let result = function
            .call([6.0.into(), 7.0.into()])
            .into_number()
            .unwrap();
        assert_eq!(result, 13.);
    }
}
