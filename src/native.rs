use std::{any::Any, mem::{forget, transmute, MaybeUninit}};

use crate::{eval, global_get, global_set, Function, Object, Value};

#[derive(Clone, Copy)]
struct FatPointer {
    data_pointer: *const u8,
    vtable_pointer: *const u8,
}

fn registry() -> Object {
    let registry = if let Some(registry) = global_get("__finalization_registry").into_object() {
        registry
    } else {
        let cleanup = Function::new_static(|args| {
            let object = args.get(0).into_object().unwrap();
            let Some(data) = object.get("_data").into_number() else {
                return Value::Undefined;
            };
            let Some(vtable) = object.get("_vtable").into_number() else {
                return Value::Undefined;
            };
            let fat_pointer = FatPointer {
                data_pointer: data as usize as *mut u8,
                vtable_pointer: vtable as usize as *mut u8,
            };
            unsafe {
                let any: *mut MaybeUninit<Box<dyn Any>> = &mut transmute(fat_pointer);
                (*any).assume_init_drop();
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

fn add_drop(object: Object) -> Object {
    let registry = registry();
    let register = registry.get("register").into_function().unwrap();
    let value = Object::new();
    value.set("_data", object.get("_data"));
    value.set("_vtable", object.get("_vtable"));
    let token = Object::new();
    register.call_with(
        registry.clone(),
        [object.into(), value.into(), token.clone().into()],
    );
    token
}

fn remove_drop(token: Object) {
    let registry = registry();
    let unregister = registry.get("unregister").into_function().unwrap();
    unregister.call_with(registry.clone(), [token.into()]);
}

pub fn wrap<T: 'static>(value: T) -> Object {
    let any: Box<dyn Any> = Box::new(value);
    let fat_pointer = unsafe { *(&any as *const _ as *const FatPointer) };
    forget(any);
    let object = Object::new();
    object.set("_valid", true);
    object.set("_data", fat_pointer.data_pointer as usize as f64);
    object.set("_vtable", fat_pointer.vtable_pointer as usize as f64);
    let token = add_drop(object.clone());
    object.set("_token", token);
    object
}

pub fn get<'a, T: 'static>(object: &'a Object) -> Option<&'a T> {
    if object.get("_valid").into_boolean().unwrap() {
        let fat_pointer = FatPointer {
            data_pointer: object.get("_data").into_number()? as usize as *mut u8,
            vtable_pointer: object.get("_vtable").into_number()? as usize as *mut u8,
        };
        let any: *const MaybeUninit<Box<dyn Any>> = unsafe { &transmute(fat_pointer) };
        unsafe {
            (*any).assume_init_ref().downcast_ref::<T>()
        }
    } else {
        None
    }
}

pub fn take<T: 'static>(object: Object) -> Option<Box<T>> {
    if object.get("_valid").into_boolean().unwrap() {
        let fat_pointer = FatPointer {
            data_pointer: object.get("_data").into_number()? as usize as *mut u8,
            vtable_pointer: object.get("_vtable").into_number()? as usize as *mut u8,
        };
        unsafe {
            let any: *const MaybeUninit<Box<dyn Any>> = &transmute(fat_pointer);
            if (*any).assume_init_ref().is::<T>() {
                object.set("_valid", false);
                remove_drop(object.get("_token").into_object().unwrap());
                let any: MaybeUninit<Box<dyn Any>> = transmute(fat_pointer);
                any.assume_init().downcast::<T>().ok()
            } else {
                None
            }
        }
    } else {
        None
    }
}
