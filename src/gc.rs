use std::{any::Any, mem::{forget, MaybeUninit}};

use crate::{eval, global_get, global_set, Function, Object, Value};

// TODO: it's not clear if FinalizationRegistry works natively in v8
// maybe need another solution there

#[derive(Clone, Copy)]
struct FatPointer {
    data_pointer: *mut u8,
    vtable_pointer: *mut u8,
}

impl FatPointer {
    fn new(mut any: Box<dyn Any>) -> FatPointer {
        let fat_pointer = unsafe { *(&mut any as *mut _ as *mut FatPointer) };
        forget(any);
        fat_pointer
    }

    fn from_object(object: Object) -> Self {
        FatPointer {
            data_pointer: object.get("_data").into_number().unwrap() as usize as *mut u8,
            vtable_pointer: object.get("_vtable").into_number().unwrap() as usize as *mut u8,
        }
    }

    fn to_object(self) -> Object {
        let object = Object::new();
        object.set("_data", self.data_pointer as usize as f64);
        object.set("_vtable", self.vtable_pointer as usize as f64);
        object
    }

    unsafe fn into_any(self) -> Box<dyn Any> {
        let mut any: MaybeUninit<Box<dyn Any>> = MaybeUninit::uninit();
        let fat_ptr = &mut any as *mut _ as *mut FatPointer;
        unsafe {
            (*fat_ptr).data_pointer = self.data_pointer;
            (*fat_ptr).vtable_pointer = self.vtable_pointer;
        }
        any.assume_init()
    }

    unsafe fn drop(self) {
        drop(self.into_any())
    }
}

fn registry() -> Object {
    let registry = if let Some(registry) = global_get("__finalization_registry").into_object() {
        registry
    } else {
        let cleanup = Function::new(|args| {
            let value = args.get(0).into_object().unwrap();
            unsafe {
                FatPointer {
                    data_pointer: value.get("_data").into_number().unwrap() as usize as *mut u8,
                    vtable_pointer: value.get("_vtable").into_number().unwrap() as usize as *mut u8,
                }
                .drop();
            }
            Value::Undefined
        });
        let finalization_registry = eval("FinalizationRegistry").into_function().unwrap();
        let registry = finalization_registry.new_instance([cleanup.into()]);
        global_set("__finalization_registry", registry.clone());
        registry
    };
    registry
}

fn add_drop(object: Object, fat_pointer: FatPointer) -> Object {
    let registry = registry();
    let register = registry.get("register").into_function().unwrap();
    let value = Object::new();
    value.set("_data", fat_pointer.data_pointer as usize as f64);
    value.set("_vtable", fat_pointer.vtable_pointer as usize as f64);
    let token = Object::new();
    register.call_with(
        registry.clone().into(),
        [object.into(), value.into(), token.clone().into()],
    );
    token
}

fn remove_drop(token: Object) {
    let registry = registry();
    let unregister = registry.get("unregister").into_function().unwrap();
    unregister.call_with(registry.clone().into(), [token.into()]);
}

pub struct GarbageCollected(Box<dyn Any>);

impl GarbageCollected {
    pub fn new<T: 'static>(value: T) -> Self {
        Self(Box::new(value))
    }

    pub fn from_js(object: Object) -> Option<GarbageCollected> {
        if object.get("_valid").into_boolean().unwrap() {
            object.set("_valid", false);
            let token = object.get("_token").into_object().unwrap();
            remove_drop(token);
            Some(GarbageCollected(unsafe {
                FatPointer::from_object(object).into_any()
            }))
        } else {
            None
        }
    }

    pub fn into_js(self) -> Object {
        let fat_pointer = FatPointer::new(self.0);
        let object = fat_pointer.to_object();
        object.set("_valid", true);
        let token = add_drop(object.clone(), fat_pointer);
        object.set("_token", token.clone());
        object
    }

    pub fn take(self) -> Box<dyn Any> {
        self.0
    }
}
