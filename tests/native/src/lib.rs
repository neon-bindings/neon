#[macro_use]
extern crate neon;

mod js {
    pub mod strings;
    pub mod numbers;
    pub mod arrays;
    pub mod objects;
    pub mod functions;
    pub mod classes;
    pub mod tasks;
}

use js::strings::return_js_string;
use js::numbers::*;
use js::arrays::*;
use js::objects::*;
use js::functions::*;
use js::classes::*;
use js::tasks::*;

use neon::mem::Handle;
use neon::js::{JsFunction, Object};
use neon::js::class::{Class, JsClass};

register_module!(m, {
    m.export("return_js_string", return_js_string)?;

    m.export("return_js_number", return_js_number)?;
    m.export("return_large_js_number", return_large_js_number)?;
    m.export("return_negative_js_number", return_negative_js_number)?;
    m.export("return_float_js_number", return_float_js_number)?;
    m.export("return_negative_float_js_number", return_negative_float_js_number)?;
    m.export("accept_and_return_js_number", accept_and_return_js_number)?;
    m.export("accept_and_return_large_js_number", accept_and_return_large_js_number)?;
    m.export("accept_and_return_float_js_number", accept_and_return_float_js_number)?;
    m.export("accept_and_return_negative_js_number", accept_and_return_negative_js_number)?;

    m.export("return_js_array", return_js_array)?;
    m.export("return_js_array_with_integer", return_js_array_with_integer)?;
    m.export("return_js_array_with_string", return_js_array_with_string)?;

    m.export("return_js_object", return_js_object)?;
    m.export("return_js_object_with_integer", return_js_object_with_integer)?;
    m.export("return_js_object_with_string", return_js_object_with_string)?;
    m.export("return_js_object_with_mixed_content", return_js_object_with_mixed_content)?;

    m.export("return_js_function", return_js_function)?;
    m.export("call_js_function", call_js_function)?;
    m.export("construct_js_function", construct_js_function)?;

    m.export("check_string_and_number", check_string_and_number)?;

    m.export("perform_async_task", perform_async_task)?;
    m.export("perform_failing_task", perform_failing_task)?;

    m.export("panic", panic)?;
    m.export("panic_after_throw", panic_after_throw)?;

    let class: Handle<JsClass<JsUser>> = JsUser::class(m.scope)?;
    let constructor: Handle<JsFunction<JsUser>> = class.constructor(m.scope)?;
    m.exports.set("User", constructor)?;

    let class: Handle<JsClass<JsPanickyAllocator>> = JsPanickyAllocator::class(m.scope)?;
    let constructor: Handle<JsFunction<JsPanickyAllocator>> = class.constructor(m.scope)?;
    m.exports.set("PanickyAllocator", constructor)?;

    let class: Handle<JsClass<JsPanickyConstructor>> = JsPanickyConstructor::class(m.scope)?;
    let constructor: Handle<JsFunction<JsPanickyConstructor>> = class.constructor(m.scope)?;
    m.exports.set("PanickyConstructor", constructor)?;

    Ok(())
});
