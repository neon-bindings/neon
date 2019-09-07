use neon::prelude::*;
use neon::register_module;

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    Ok(cx.string("Hello, World!"))
}

register_module!(mut cx, {
    cx.export_function("hello", hello)?;

    Ok(())
});
