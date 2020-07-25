use neon::prelude::*;

mod js {
    pub mod coercions;
    pub mod errors;
    pub mod functions;
    pub mod objects;
}

use js::coercions::*;
use js::errors::*;
use js::functions::*;
use js::objects::*;

register_module!(|mut cx| {
    let greeting = cx.string("Hello, World!");
    let greeting_copy = greeting.value(&mut cx);
    let greeting_copy = cx.string(greeting_copy);

    cx.export_value("greeting", greeting)?;
    cx.export_value("greetingCopy", greeting_copy)?;

    // Global singletons.
    let undefined = cx.undefined();
    let null = cx.null();
    let b_true = cx.boolean(true);
    let b_false = cx.boolean(false);

    assert_eq!(b_true.value(&mut cx), true);
    assert_eq!(b_false.value(&mut cx), false);

    cx.export_value("undefined", undefined)?;
    cx.export_value("null", null)?;
    cx.export_value("true", b_true)?;
    cx.export_value("false", b_false)?;

    let one = cx.number(1);
    let two = cx.number(2.1);
    assert_eq!(one.value(&mut cx), 1.0);
    assert_eq!(two.value(&mut cx), 2.1);
    cx.export_value("one", one)?;
    cx.export_value("two", two)?;

    // Plain objects.
    let rust_created = cx.empty_object();
    {
        let a = cx.number(1);
        // set at name
        rust_created.set(&mut cx, "a", a)?;
        // set at index
        rust_created.set(&mut cx, 0, a)?;
    }
    {
        let whatever = cx.boolean(true);
        rust_created.set(&mut cx, "whatever", whatever)?;
    }

    assert_eq!({
        let v: Handle<JsNumber> = rust_created.get(&mut cx, "a")?.downcast_or_throw(&mut cx)?;
        v.value(&mut cx)
    }, 1.0f64);
    assert_eq!({
        let v: Handle<JsNumber> = rust_created.get(&mut cx, 0)?.downcast_or_throw(&mut cx)?;
        v.value(&mut cx)
    }, 1.0f64);
    assert_eq!({
        let v: Handle<JsBoolean> = rust_created.get(&mut cx, "whatever")?.downcast_or_throw(&mut cx)?;
        v.value(&mut cx)
    }, true);

    let property_names = rust_created.get_own_property_names(&mut cx)?
        .to_vec(&mut cx)?
        .into_iter()
        .map(|value| {
            let string: Handle<JsString> = value.downcast_or_throw(&mut cx)?;
            Ok(string.value(&mut cx))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(property_names, &["0", "a", "whatever"]);

    cx.export_value("rustCreated", rust_created)?;

    fn add1(mut cx: FunctionContext) -> JsResult<JsNumber> {
        let x = cx.argument::<JsNumber>(0)?.value(&mut cx);
        Ok(cx.number(x + 1.0))
    }

    cx.export_function("add1", add1)?;

    cx.export_function("return_js_function", return_js_function)?;
    cx.export_function("call_js_function", call_js_function)?;
    cx.export_function("construct_js_function", construct_js_function)?;
    cx.export_function("num_arguments", num_arguments)?;
    cx.export_function("return_this", return_this)?;
    cx.export_function("require_object_this", require_object_this)?;
    cx.export_function("is_argument_zero_some", is_argument_zero_some)?;
    cx.export_function("require_argument_zero_string", require_argument_zero_string)?;
    cx.export_function("check_string_and_number", check_string_and_number)?;
    cx.export_function("execute_scoped", execute_scoped)?;
    cx.export_function("compute_scoped", compute_scoped)?;

    cx.export_function("to_string", to_string)?;

    cx.export_function("return_js_global_object", return_js_global_object)?;
    cx.export_function("return_js_object", return_js_object)?;
    cx.export_function("return_js_object_with_number", return_js_object_with_number)?;
    cx.export_function("return_js_object_with_string", return_js_object_with_string)?;
    cx.export_function("return_js_object_with_mixed_content", return_js_object_with_mixed_content)?;

    cx.export_function("return_array_buffer", return_array_buffer)?;
    cx.export_function("read_array_buffer_with_lock", read_array_buffer_with_lock)?;
    cx.export_function("read_array_buffer_with_borrow", read_array_buffer_with_borrow)?;
    cx.export_function("write_array_buffer_with_lock", write_array_buffer_with_lock)?;
    cx.export_function("write_array_buffer_with_borrow_mut", write_array_buffer_with_borrow_mut)?;
    cx.export_function("return_buffer", return_buffer)?;
    cx.export_function("read_buffer_with_lock", read_buffer_with_lock)?;
    cx.export_function("read_buffer_with_borrow", read_buffer_with_borrow)?;
    cx.export_function("write_buffer_with_lock", write_buffer_with_lock)?;
    cx.export_function("write_buffer_with_borrow_mut", write_buffer_with_borrow_mut)?;

    cx.export_function("new_error", new_error)?;
    cx.export_function("new_type_error", new_type_error)?;
    cx.export_function("new_range_error", new_range_error)?;
    cx.export_function("throw_error", throw_error)?;

    cx.export_function("panic", panic)?;
    cx.export_function("panic_after_throw", panic_after_throw)?;

    cx.export_function("throw_and_catch", throw_and_catch)?;
    cx.export_function("call_and_catch", call_and_catch)?;

    fn call_get_own_property_names(mut cx: FunctionContext) -> JsResult<JsArray> {
        let object = cx.argument::<JsObject>(0)?;
        object.get_own_property_names(&mut cx)
    }

    cx.export_function("get_own_property_names", call_get_own_property_names)?;

    Ok(())
});
