use std::mem::transmute;

static mut GLOBAL: Option<Global> = None;

#[derive(Default)]
struct Global {
    owned_isolate: Option<v8::OwnedIsolate>,
    handle_scope: Option<v8::HandleScope<'static, ()>>,
    context: Option<v8::Local<'static, v8::Context>>,
    context_scope: Option<v8::ContextScope<'static, v8::HandleScope<'static, v8::Context>>>,
    scope_stack: Vec<*mut v8::HandleScope<'static, v8::Context>>,
}

unsafe fn global() -> &'static mut Global {
    GLOBAL.get_or_insert_with(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
        let mut global = Global::default();
        {
            let global: &'static mut Global = transmute(&mut global);
            let Global {
                owned_isolate,
                handle_scope,
                context,
                context_scope,
                scope_stack,
            } = global;
            *owned_isolate = Some(v8::Isolate::new(v8::CreateParams::default()));
            *handle_scope = Some(v8::HandleScope::new(owned_isolate.as_mut().unwrap()));
            *context = Some(v8::Context::new(handle_scope.as_mut().unwrap()));
            *context_scope = Some(v8::ContextScope::new(
                handle_scope.as_mut().unwrap(),
                *context.as_mut().unwrap(),
            ));
            *scope_stack = Vec::new();
        }
        global
    })
}

pub(crate) fn scope(
) -> &'static mut v8::ContextScope<'static, v8::HandleScope<'static, v8::Context>> {
    unsafe {
        let global = global();
        if let Some(scope) = global.scope_stack.last_mut() {
            transmute(*scope)
        } else {
            global.context_scope.as_mut().unwrap()
        }
    }
}

pub fn push_scope(scope: &mut v8::HandleScope<'_>) {
    unsafe {
        let global = GLOBAL.get_or_insert_with(|| Global::default());
        global.scope_stack.push(transmute(scope));
    }
}

pub fn pop_scope() {
    unsafe {
        let global = GLOBAL.get_or_insert_with(|| Global::default());
        global.scope_stack.pop();
    }
}