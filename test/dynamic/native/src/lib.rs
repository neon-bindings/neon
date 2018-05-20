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

register_module!(mut cx, {
    cx.export_function("return_js_string", return_js_string)?;

    cx.export_function("return_js_number", return_js_number)?;
    cx.export_function("return_large_js_number", return_large_js_number)?;
    cx.export_function("return_negative_js_number", return_negative_js_number)?;
    cx.export_function("return_float_js_number", return_float_js_number)?;
    cx.export_function("return_negative_float_js_number", return_negative_float_js_number)?;
    // DEPRECATE(0.2)
    cx.export_function("return_js_integer", return_js_integer)?;
    cx.export_function("accept_and_return_js_number", accept_and_return_js_number)?;
    cx.export_function("accept_and_return_large_js_number", accept_and_return_large_js_number)?;
    cx.export_function("accept_and_return_float_js_number", accept_and_return_float_js_number)?;
    cx.export_function("accept_and_return_negative_js_number", accept_and_return_negative_js_number)?;
    // DEPRECATE(0.2)
    cx.export_function("accept_and_return_js_integer", accept_and_return_js_integer)?;

    cx.export_function("return_js_array", return_js_array)?;
    cx.export_function("return_js_array_with_number", return_js_array_with_number)?;
    cx.export_function("return_js_array_with_string", return_js_array_with_string)?;

    cx.export_function("return_js_global_object", return_js_global_object)?;
    cx.export_function("return_js_object", return_js_object)?;
    cx.export_function("return_js_object_with_number", return_js_object_with_number)?;
    cx.export_function("return_js_object_with_string", return_js_object_with_string)?;
    cx.export_function("return_js_object_with_mixed_content", return_js_object_with_mixed_content)?;

    cx.export_function("return_js_function", return_js_function)?;
    cx.export_function("call_js_function", call_js_function)?;
    cx.export_function("construct_js_function", construct_js_function)?;

    cx.export_function("check_string_and_number", check_string_and_number)?;

    cx.export_function("perform_async_task", perform_async_task)?;
    cx.export_function("perform_failing_task", perform_failing_task)?;

    cx.export_function("panic", panic)?;
    cx.export_function("panic_after_throw", panic_after_throw)?;

    cx.export_class::<JsUser>("User")?;
    cx.export_class::<JsPanickyAllocator>("PanickyAllocator")?;
    cx.export_class::<JsPanickyConstructor>("PanickyConstructor")?;

    Ok(())
});
