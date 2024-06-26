use crate::{native, AsObject, Exception, Object, Value};

#[derive(Clone, PartialEq)]
pub struct Function {
    #[cfg(not(target_arch = "wasm32"))]
    function: v8::Global<v8::Function>,
    #[cfg(target_arch = "wasm32")]
    function: js_sys::Function,
}

impl Function {
    pub fn new<F: Fn(Args) -> Result<R, Exception> + 'static, R: Into<Value> + 'static>(
        body: F,
    ) -> Self {
        Self::new_with_data(Value::Undefined, body)
    }

    pub fn new_with_data<
        F: Fn(Args) -> Result<R, Exception> + 'static,
        R: Into<Value> + 'static,
    >(
        data: impl Into<Value>,
        body: F,
    ) -> Self {
        let body_box: Box<dyn Fn(Args) -> Result<R, Exception>> = Box::new(body);
        let closure = native::wrap(body_box);
        let data_arr = Value::from(vec![closure.into(), data.into()]);
        let function = Self::new_static_with_data(data_arr, |mut args: Args| {
            let data_arr = args.data().into_array().unwrap();
            let closure = data_arr.get(0).into_object().unwrap();
            let data = data_arr.get(1);
            let closure =
                native::get::<Box<dyn Fn(Args) -> Result<R, Exception>>>(&closure).unwrap();
            args.data = data;
            match (closure)(args).into() {
                Ok(value) => Ok(value.into()),
                Err(value) => Err(Value::from(value)),
            }
        });
        function
    }

    pub fn new_static(body: fn(Args) -> Result<Value, Value>) -> Self {
        Self::new_static_with_data(Value::Undefined, body)
    }

    pub fn new_static_with_data(
        data: impl Into<Value>,
        body: fn(Args) -> Result<Value, Value>,
    ) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let data_arr = crate::Array::new();
            data_arr.push(body as usize as f64);
            data_arr.push(data);
            let function = v8::Function::builder(
                |v8_scope: &mut v8::HandleScope<'_>,
                 v8_args: v8::FunctionCallbackArguments<'_>,
                 mut v8_ret: v8::ReturnValue<'_>| {
                    crate::v8::push_scope(v8_scope);
                    let data_arr = Value::from(v8_args.data()).into_array().unwrap();
                    let body_ptr = data_arr.get(0).into_number().unwrap();
                    let f: fn(Args) -> Result<Value, Value> =
                        unsafe { std::mem::transmute(body_ptr as usize) };
                    let data = data_arr.get(1);
                    let this = Value::from(Object::from(v8_args.this()));
                    let mut args = Args {
                        this,
                        data,
                        args: vec![],
                    };
                    for i in 0..v8_args.length() {
                        args.args.push(Value::from(v8_args.get(i)));
                    }
                    match f(args) {
                        Ok(value) => {
                            let value = v8::Local::<v8::Value>::from(value);
                            crate::v8::pop_scope();
                            v8_ret.set(value);
                        }
                        Err(err) => {
                            v8_scope.throw_exception(v8::Local::<v8::Value>::from(err));
                            crate::v8::pop_scope();
                        }
                    }
                },
            )
            .data(v8::Local::<v8::Value>::from(v8::Local::<v8::Array>::from(
                data_arr,
            )))
            .build(scope)
            .unwrap();
            Self {
                function: v8::Global::new(scope, function),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let id = format!("__staticfn_{}", body as usize);
            let inner_function = if let Value::Function(function) = crate::global_get(&id) {
                function
            } else {
                let function = {
                    use wasm_bindgen::{closure::Closure, JsValue};
                    let bindgen_closure = Closure::<
                        dyn Fn(JsValue, JsValue, JsValue) -> Result<JsValue, JsValue>,
                    >::new(
                        move |js_this: JsValue, js_data: JsValue, js_args: JsValue| {
                            let this = Value::from(js_this);
                            let data = Value::from(js_data);
                            let mut args = Args {
                                this,
                                data,
                                args: vec![],
                            };
                            let js_args_array: js_sys::Array = js_args.into();
                            for i in 0..js_args_array.length() {
                                args.args.push(Value::from(js_args_array.get(i)));
                            }
                            match body(args) {
                                Ok(value) => Ok(JsValue::from(value)),
                                Err(err) => Err(JsValue::from(err)),
                            }
                        },
                    );
                    let closure = Value::from(JsValue::from(bindgen_closure.as_ref()))
                        .into_function()
                        .unwrap();
                    bindgen_closure.forget();
                    closure
                };
                crate::global_set(&id, function.clone());
                function
            };
            let js_wrapper = r#"function wrapper() {
                return wrapper.__fn.apply(null, [this, wrapper.__data, Array.from(arguments)]);
            }"#;
            // TODO: add define_property to Object
            let function = crate::eval(&format!(
                "{}; Object.defineProperty(wrapper, \"name\", {{ value: \"\" }}); wrapper",
                js_wrapper
            ))
            .unwrap()
            .into_function()
            .unwrap();
            function.as_object().set("__fn", inner_function);
            function.as_object().set("__data", data);
            function
        }
    }

    pub fn call(&self, args: impl IntoIterator<Item = Value>) -> Result<Value, Value> {
        self.call_with(Value::Undefined, args)
    }

    pub fn call_with(
        &self,
        receiver: impl Into<Value>,
        args: impl IntoIterator<Item = Value>,
    ) -> Result<Value, Value> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let function = v8::Local::new(scope, self.function.clone());
            let receiver = v8::Local::<v8::Value>::from(receiver.into());
            let args = args
                .into_iter()
                .map(|value| v8::Local::<v8::Value>::from(value))
                .collect::<Vec<_>>();
            let result = {
                let scope = &mut v8::TryCatch::new(scope);
                crate::v8::push_scope(scope);
                if let Some(ret) = function.call(scope, receiver, &args) {
                    Ok(Value::from(ret))
                } else {
                    // TODO: don't unwrap
                    let exception = scope.exception().unwrap();
                    Err(Value::from(exception))
                }
            };
            crate::v8::pop_scope();
            result
        }
        #[cfg(target_arch = "wasm32")]
        {
            let receiver = wasm_bindgen::JsValue::from(receiver.into());
            let array = js_sys::Array::new();
            for arg in args {
                array.push(&wasm_bindgen::JsValue::from(arg));
            }
            match self.function.apply(&receiver, &array) {
                Ok(value) => Ok(Value::from(value)),
                Err(value) => Err(Value::from(value)),
            }
        }
    }

    pub fn new_instance(&self, args: impl IntoIterator<Item = Value>) -> Result<Object, Value> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let function = v8::Local::new(scope, self.function.clone());
            let args = args
                .into_iter()
                .map(|value| v8::Local::<v8::Value>::from(value))
                .collect::<Vec<_>>();
            let result = {
                let scope = &mut v8::TryCatch::new(scope);
                crate::v8::push_scope(scope);
                if let Some(ret) = function.new_instance(scope, &args) {
                    Ok(Object::from(ret))
                } else {
                    // TODO: don't unwrap
                    let exception = scope.exception().unwrap();
                    Err(Value::from(exception))
                }
            };
            crate::v8::pop_scope();
            result
        }
        #[cfg(target_arch = "wasm32")]
        {
            let array = js_sys::Array::new();
            for arg in args {
                array.push(&wasm_bindgen::JsValue::from(arg));
            }
            match js_sys::Reflect::construct(&self.function, &array) {
                Ok(object) => Ok(Object::from(js_sys::Object::from(object))),
                Err(err) => Err(Value::from(err)),
            }
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

impl std::fmt::Debug for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = self
            .as_object()
            .get("name")
            .into_string()
            .unwrap_or_else(|| String::new());
        if name.is_empty() {
            write!(f, "ƒ()")
        } else {
            write!(f, "{}()", name)
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
impl From<Function> for js_sys::Function {
    fn from(function: Function) -> Self {
        js_sys::Function::from(function.function)
    }
}

#[derive(Debug, Clone)]
pub struct Args {
    this: Value,
    data: Value,
    args: Vec<Value>,
}

impl Args {
    pub fn this(&self) -> Value {
        self.this.clone()
    }

    pub fn this_ref(&self) -> &Value {
        &self.this
    }

    pub fn data(&self) -> Value {
        self.data.clone()
    }

    pub fn data_ref(&self) -> &Value {
        &self.data
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
    fn call_static_function() {
        let function = Function::new_static_with_data(Value::Number(1234.), |args| {
            assert_eq!(args.data(), Value::Number(1234.));
            let a = args.get(0).into_number().unwrap();
            let b = args.get(1).into_number().unwrap();
            Ok(Value::from(a + b))
        });
        let result = function
            .call([6.0.into(), 7.0.into()])
            .unwrap()
            .into_number()
            .unwrap();
        assert_eq!(result, 13.);
    }

    #[test]
    fn call_js_function() {
        let function = eval("function test(a, b) { return a + b; }; test")
            .unwrap()
            .into_function()
            .unwrap();
        let result = function
            .call([6.0.into(), 7.0.into()])
            .unwrap()
            .into_number()
            .unwrap();
        assert_eq!(result, 13.);
    }

    #[test]
    fn call_js_function_with_exception() {
        let function = eval("function test(a, b) { throw a + b; }; test")
            .unwrap()
            .into_function()
            .unwrap();
        let result = function
            .call([6.0.into(), 7.0.into()])
            .unwrap_err()
            .into_number()
            .unwrap();
        assert_eq!(result, 13.);
    }

    #[test]
    fn call_function() {
        let function = Function::new(|_| Ok(true));
        let result = function.call([]).unwrap();
        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn call_function_with_data() {
        let function = Function::new_with_data(777., |args| Ok(args.data()));
        let result = function.call([]).unwrap();
        assert_eq!(result, Value::Number(777.));
    }

    #[test]
    fn call_function_with_capture() {
        let function = Function::new(|args| {
            let name = args.get(0).into_string().unwrap();
            Ok(Value::from(Function::new(move |_| Ok(name.clone()))))
        });
        let result = function
            .call(["Bob".into()])
            .unwrap()
            .into_function()
            .unwrap();
        assert_eq!(result.call([]).unwrap(), Value::String("Bob".to_owned()));
    }

    #[test]
    fn call_new() {
        let function = Function::new(|args| {
            let this = args.this().into_object().unwrap();
            this.set("x", 1.);
            this.set("y", 2.);
            Ok(())
        });
        let result = function.new_instance([]).unwrap();
        assert_eq!(result.get("x"), Value::Number(1.));
        assert_eq!(result.get("y"), Value::Number(2.));
    }

    #[test]
    fn call_new_with_exception() {
        let function = Function::new(|_| Err::<Value, _>(Value::String("nope".to_owned()).into()));
        let result = function.new_instance([]).unwrap_err();
        assert_eq!(result, Value::String("nope".to_owned()));
    }

    #[test]
    fn call_js_new() {
        let function = eval("function test() { this.x = 1; this.y = 2; }; test")
            .unwrap()
            .into_function()
            .unwrap();
        let result = function.new_instance([]).unwrap();
        assert_eq!(result.get("x"), Value::Number(1.));
        assert_eq!(result.get("y"), Value::Number(2.));
    }

    #[test]
    fn call_js_new_with_exception() {
        let function = eval("function test() { throw \"nope\"; }; test")
            .unwrap()
            .into_function()
            .unwrap();
        let result = function.new_instance([]).unwrap_err();
        assert_eq!(result, Value::String("nope".to_owned()));
    }
}
