use crate::{AsObject, Object, Value};

#[derive(Clone, PartialEq)]
pub struct Array {
    #[cfg(not(target_arch = "wasm32"))]
    array: v8::Global<v8::Array>,
    #[cfg(target_arch = "wasm32")]
    array: js_sys::Array,
}

impl Array {
    pub fn new() -> Self {
        Self::new_with_length(0)
    }

    pub fn new_with_length(length: u32) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let array = v8::Array::new(scope, length as i32);
            Self {
                array: v8::Global::new(scope, array),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                array: js_sys::Array::new_with_length(length),
            }
        }
    }

    pub fn length(&self) -> u32 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let array = v8::Local::new(scope, self.array.clone());
            array.length()
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.array.length()
        }
    }

    pub fn get(&self, index: u32) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let array = v8::Local::new(scope, self.array.clone());
            let key = v8::Number::new(scope, index as f64);
            if let Some(value) = array.get(scope, key.into()) {
                Value::from(value)
            } else {
                Value::Undefined
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Value::from(self.array.get(index))
        }
    }

    pub fn set(&self, index: u32, value: impl Into<Value>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let array = v8::Local::new(scope, self.array.clone());
            let key = v8::Number::new(scope, index as f64);
            let value = v8::Local::<v8::Value>::from(value.into());
            array.set(scope, key.into(), value);
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.array
                .set(index, wasm_bindgen::JsValue::from(value.into()))
        }
    }

    pub fn push(&self, value: impl Into<Value>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let array = v8::Local::new(scope, self.array.clone());
            let length = array.length();
            let key = v8::Local::<v8::Value>::from(v8::Number::new(scope, length as f64));
            let value = v8::Local::<v8::Value>::from(value.into());
            array.set(scope, key.into(), value);
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.array.push(&wasm_bindgen::JsValue::from(value.into()));
        }
    }
}

impl AsObject for Array {
    fn as_object(&self) -> Object {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let local = v8::Local::new(scope, self.array.clone());
            Object::from(v8::Local::<v8::Object>::from(local))
        }
        #[cfg(target_arch = "wasm32")]
        {
            Object::from(js_sys::Object::from(self.array.clone()))
        }
    }
}

impl std::fmt::Debug for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Array {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[")?;
        for i in 0..self.length() {
            if i != 0 {
                write!(f, ", ")?;
            }
                write!(f, "{}", self.get(i))?;
        }
        write!(f, "]")
    }
}

impl From<Vec<Value>> for Array {
    fn from(vec: Vec<Value>) -> Self {
        let array = Array::new();
        for value in vec {
            array.push(value);
        }
        array
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<v8::Local<'_, v8::Array>> for Array {
    fn from(value: v8::Local<v8::Array>) -> Self {
        let scope = crate::v8::scope();
        Self {
            array: v8::Global::new(scope, value),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Array> for v8::Local<'_, v8::Array> {
    fn from(value: Array) -> Self {
        let scope = crate::v8::scope();
        v8::Local::new(scope, &value.array)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<js_sys::Array> for Array {
    fn from(array: js_sys::Array) -> Self {
        Self { array }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Array> for js_sys::Array {
    fn from(array: Array) -> Self {
        js_sys::Array::from(&array.array)
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::{Array, Object};

    #[test]
    fn length() {
        let array = Array::new();
        assert_eq!(array.length(), 0);
        array.push(0.0);
        array.push(0.0);
        array.push(0.0);
        assert_eq!(array.length(), 3);
    }

    #[test]
    fn set_get() {
        let array = Array::new();
        array.set(0, "foo");
        array.set(1, 999.);
        array.set(2, Object::new());
        assert!(array.get(0).is_string());
        assert!(array.get(1).is_number());
        assert!(array.get(2).is_object());
        assert!(array.get(3).is_undefined());
    }

    #[test]
    fn push() {
        let array = Array::from(vec![0.0.into(), 1.0.into()]);
        array.push(2.);
        array.push(3.);
        assert_eq!(array.get(0).into_number().unwrap(), 0.);
        assert_eq!(array.get(1).into_number().unwrap(), 1.);
        assert_eq!(array.get(2).into_number().unwrap(), 2.);
        assert_eq!(array.get(3).into_number().unwrap(), 3.);
    }

    #[test]
    fn from_vec() {
        let array = Array::from(vec![
            "foo".into(),
            999.0.into(),
            Object::new().into(),
        ]);
        assert!(array.get(0).is_string());
        assert!(array.get(1).is_number());
        assert!(array.get(2).is_object());
        assert!(array.get(3).is_undefined());
    }
}
