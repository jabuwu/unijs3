use crate::{eval, Value};

pub trait AsObject {
    fn as_object(&self) -> Object;
}

#[derive(Clone, PartialEq)]
pub struct Object {
    #[cfg(not(target_arch = "wasm32"))]
    object: v8::Global<v8::Object>,
    #[cfg(target_arch = "wasm32")]
    object: js_sys::Object,
}

impl Object {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Object::new(scope);
            Self {
                object: v8::Global::new(scope, object),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                object: js_sys::Object::new(),
            }
        }
    }

    pub fn get(&self, key: &str) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            let name = v8::String::new(scope, key).unwrap();
            let value = object.get(scope, name.into());
            if let Some(value) = value {
                Value::from(value)
            } else {
                Value::Undefined
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let key = wasm_bindgen::JsValue::from_str(key);
            js_sys::Reflect::get(&self.object, &key)
                .map(|value| Value::from(value))
                .unwrap_or_else(|_| Value::Undefined)
        }
    }

    pub fn set(&self, key: &str, value: impl Into<Value>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            let key = v8::String::new(scope, key).unwrap();
            let value = v8::Local::<v8::Value>::from(value.into());
            object.set(scope, key.into(), value).unwrap();
        }
        #[cfg(target_arch = "wasm32")]
        {
            let key = wasm_bindgen::JsValue::from_str(key);
            // TODO: don't unwrap
            js_sys::Reflect::set(
                &self.object,
                &key,
                &wasm_bindgen::JsValue::from(value.into()),
            )
            .unwrap();
        }
    }

    pub fn delete(&self, key: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            let key = v8::String::new(scope, key).unwrap();
            object.delete(scope, key.into());
        }
        #[cfg(target_arch = "wasm32")]
        {
            let key = wasm_bindgen::JsValue::from_str(key);
            // TODO: don't unwrap
            js_sys::Reflect::delete_property(&self.object, &key).unwrap();
        }
    }

    pub fn keys(&self) -> Vec<String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            let names = object
                .get_own_property_names(scope, v8::GetPropertyNamesArgs::default())
                .unwrap();
            let mut keys = vec![];
            for i in 0..names.length() {
                let i_key = v8::Number::new(scope, i as f64);
                if let Some(name) = names.get(scope, i_key.into()) {
                    let name = Value::from(name);
                    if let Some(name) = name.into_string() {
                        keys.push(name);
                    }
                }
            }
            keys
        }
        #[cfg(target_arch = "wasm32")]
        {
            let mut keys = vec![];
            let object_keys = js_sys::Reflect::own_keys(&self.object.clone().into()).unwrap();
            for item in object_keys {
                if let Some(name) = item.as_string() {
                    keys.push(name);
                }
            }
            keys
        }
    }

    pub fn instanceof(&self, prototype: impl Into<Value>) -> bool {
        let prototype = prototype.into();
        if let Value::Object(my_prototype) = self.prototype() {
            if my_prototype.as_object().get("constructor") == prototype {
                true
            } else {
                my_prototype.as_object().instanceof(prototype)
            }
        } else {
            false
        }
    }

    pub fn prototype(&self) -> Value {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            if let Some(prototype) = object.get_prototype(scope) {
                Value::from(prototype)
            } else {
                Value::Undefined
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let prototype = js_sys::Object::get_prototype_of(&self.object);
            Value::from(Object::from(prototype))
        }
    }

    pub fn set_prototype(&self, prototype: impl Into<Value>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            let value = v8::Local::<v8::Value>::from(prototype.into());
            object.set_prototype(scope, value);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let prototype = js_sys::Object::from(wasm_bindgen::JsValue::from(prototype.into()));
            js_sys::Object::set_prototype_of(&self.object, &prototype.into());
        }
    }

    // TODO: accept PropertyDescriptor instead of value
    pub fn define_property(&self, key: &str, value: impl Into<Value>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let scope = crate::v8::scope();
            let object = v8::Local::new(scope, self.object.clone());
            let key = v8::String::new(scope, key).unwrap();
            let value = v8::Local::<v8::Value>::from(value.into());
            let property_descriptor = v8::PropertyDescriptor::new_from_value(value);
            object.define_property(scope, key.into(), &property_descriptor);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let key = wasm_bindgen::JsValue::from(Value::from(key));
            let attributes = Object::new();
            attributes.set("value", value);
            js_sys::Reflect::define_property(
                &self.object,
                &key,
                &js_sys::Object::from(attributes),
            ).unwrap();
        }
    }
}

impl AsObject for Object {
    fn as_object(&self) -> Object {
        self.clone()
    }
}

impl std::fmt::Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let error = eval("Error").unwrap();
        if self.instanceof(error) {
            let message = self.get("message").into_string().unwrap_or_else(|| String::from("??"));
            write!(f, "Error: {}", message)
        } else {
            write!(f, "{{")?;
            let keys = self.keys();
            if keys.len() > 0 {
                write!(f, " ")?;
                for i in 0..keys.len() {
                    let key = &keys[i];
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, self.get(key))?;
                }
                write!(f, " ")?;
            }
            write!(f, "}}")
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<v8::Local<'_, v8::Object>> for Object {
    fn from(value: v8::Local<v8::Object>) -> Self {
        let scope = crate::v8::scope();
        Self {
            object: v8::Global::new(scope, value),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<Object> for v8::Local<'_, v8::Object> {
    fn from(value: Object) -> Self {
        let scope = crate::v8::scope();
        v8::Local::new(scope, &value.object)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<js_sys::Object> for Object {
    fn from(object: js_sys::Object) -> Self {
        Self { object }
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Object> for js_sys::Object {
    fn from(object: Object) -> Self {
        js_sys::Object::from(object.object)
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    use crate::{Array, AsObject, Function, Object, Value};

    #[test]
    fn set_get() {
        let object = Object::new();
        object.set("foo", "bar");
        assert_eq!(object.get("foo").into_string().unwrap(), "bar");
    }

    #[test]
    fn delete() {
        let object = Object::new();
        object.set("foo", "bar");
        object.set("foo", Value::Undefined);
        assert_eq!(object.keys(), vec!["foo"]);
        object.delete("foo");
        assert_eq!(object.keys(), Vec::<String>::new());
    }

    #[test]
    fn keys() {
        let object = Object::new();
        object.set("foo", "bar");
        object.set("pi", 3.14);
        object.set("obj", Object::new());
        object.set("arr", Array::new());
        object.set("fn", Function::new(|_| Ok(())));
        assert_eq!(object.keys(), vec!["foo", "pi", "obj", "arr", "fn"]);
    }

    #[test]
    fn prototype() {
        let class = Function::new(|_| Ok(()));
        class.as_object().define_property("name", "class");

        let instance = class.new_instance([]).unwrap();
        assert_eq!(
            instance
                .prototype()
                .into_object()
                .unwrap()
                .get("constructor"),
            class.into()
        );

        let class2 = Function::new(|_| Ok(()));
        class2.as_object().define_property("name", "class2");

        instance.set_prototype(class2.clone().as_object().get("prototype"));
        assert_eq!(
            instance
                .prototype()
                .into_object()
                .unwrap()
                .get("constructor"),
            class2.into()
        );
    }
}
