#[macro_use]
extern crate neon;

use neon::prelude::*;

fn accepts_js_objs(mut ctx: FunctionContext) -> JsResult<JsString> {
  let js_object_handle: Handle<JsObject> = ctx.argument(0)?;

  let js_object = js_object_handle
    .downcast::<JsObject>()
    .unwrap_or(JsObject::new(&mut ctx));

  let rust_string = js_object.get(&mut ctx, "myProp")?
    .downcast::<JsString>()
    .unwrap_or(ctx.string(""));

  Ok(JsString::new(&mut ctx, rust_string.value()))
}

register_module!(mut ctx, {
    ctx.export_function("acceptsJsObjs", accepts_js_objs)
});
