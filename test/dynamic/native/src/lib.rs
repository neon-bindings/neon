use neon::prelude::*;

mod js {
    pub mod strings;
    pub mod numbers;
    pub mod arrays;
    pub mod objects;
    pub mod functions;
    pub mod classes;
    pub mod tasks;
    pub mod eventhandler;
}

use js::strings::return_js_string;
use js::numbers::*;
use js::arrays::*;
use js::objects::*;
use js::functions::*;
use js::classes::*;
use js::tasks::*;
use js::eventhandler::*;

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("return_js_string", return_js_string)?;

    cx.export_function("return_js_number", return_js_number)?;
    cx.export_function("return_large_js_number", return_large_js_number)?;
    cx.export_function("return_negative_js_number", return_negative_js_number)?;
    cx.export_function("return_float_js_number", return_float_js_number)?;
    cx.export_function("return_negative_float_js_number", return_negative_float_js_number)?;
    cx.export_function("accept_and_return_js_number", accept_and_return_js_number)?;
    cx.export_function("accept_and_return_large_js_number", accept_and_return_large_js_number)?;
    cx.export_function("accept_and_return_float_js_number", accept_and_return_float_js_number)?;
    cx.export_function("accept_and_return_negative_js_number", accept_and_return_negative_js_number)?;

    cx.export_function("return_js_array", return_js_array)?;
    cx.export_function("return_js_array_with_number", return_js_array_with_number)?;
    cx.export_function("return_js_array_with_string", return_js_array_with_string)?;

    cx.export_function("return_js_global_object", return_js_global_object)?;
    cx.export_function("return_js_object", return_js_object)?;
    cx.export_function("return_js_object_with_number", return_js_object_with_number)?;
    cx.export_function("return_js_object_with_string", return_js_object_with_string)?;
    cx.export_function("return_js_object_with_mixed_content", return_js_object_with_mixed_content)?;
    cx.export_function("return_array_buffer", return_array_buffer)?;
    cx.export_function("read_array_buffer_with_lock", read_array_buffer_with_lock)?;
    cx.export_function("read_array_buffer_with_borrow", read_array_buffer_with_borrow)?;
    cx.export_function("sum_array_buffer_with_borrow", sum_array_buffer_with_borrow)?;
    cx.export_function("write_array_buffer_with_lock", write_array_buffer_with_lock)?;
    cx.export_function("write_array_buffer_with_borrow_mut", write_array_buffer_with_borrow_mut)?;
    cx.export_function("increment_array_buffer_with_borrow_mut", increment_array_buffer_with_borrow_mut)?;
    cx.export_function("return_buffer", return_buffer)?;
    cx.export_function("read_buffer_with_lock", read_buffer_with_lock)?;
    cx.export_function("read_buffer_with_borrow", read_buffer_with_borrow)?;
    cx.export_function("sum_buffer_with_borrow", sum_buffer_with_borrow)?;
    cx.export_function("write_buffer_with_lock", write_buffer_with_lock)?;
    cx.export_function("write_buffer_with_borrow_mut", write_buffer_with_borrow_mut)?;
    cx.export_function("increment_buffer_with_borrow_mut", increment_buffer_with_borrow_mut)?;

    cx.export_function("return_js_function", return_js_function)?;
    cx.export_function("call_js_function", call_js_function)?;
    cx.export_function("construct_js_function", construct_js_function)?;
    cx.export_function("num_arguments", num_arguments)?;
    cx.export_function("return_this", return_this)?;
    cx.export_function("require_object_this", require_object_this)?;
    cx.export_function("is_argument_zero_some", is_argument_zero_some)?;
    cx.export_function("require_argument_zero_string", require_argument_zero_string)?;
    cx.export_function("execute_scoped", execute_scoped)?;
    cx.export_function("compute_scoped", compute_scoped)?;

    cx.export_function("check_string_and_number", check_string_and_number)?;

    cx.export_function("perform_async_task", perform_async_task)?;
    cx.export_function("perform_failing_task", perform_failing_task)?;

    cx.export_function("panic", panic)?;
    cx.export_function("panic_after_throw", panic_after_throw)?;
    cx.export_function("throw_and_catch", throw_and_catch)?;
    cx.export_function("call_and_catch", call_and_catch)?;
    cx.export_function("get_number_or_default", get_number_or_default)?;
    cx.export_function("panic_and_catch", panic_and_catch)?;
    cx.export_function("unexpected_throw_and_catch", unexpected_throw_and_catch)?;
    cx.export_function("downcast_error", downcast_error)?;

    cx.export_class::<JsEmitter>("Emitter")?;
    cx.export_class::<JsTestEmitter>("TestEmitter")?;
    cx.export_class::<JsUser>("User")?;
    cx.export_class::<JsPanickyAllocator>("PanickyAllocator")?;
    cx.export_class::<JsPanickyConstructor>("PanickyConstructor")?;

    Ok(())
}
