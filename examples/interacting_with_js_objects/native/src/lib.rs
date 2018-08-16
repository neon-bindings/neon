#[macro_use]
extern crate neon;

use neon::prelude::*;


fn accepts_js_objs(mut ctx: FunctionContext) -> JsResult<JsString> {
  let js_object_handle: Handle<JsObject> = ctx.argument(0)?;

  let js_object = match js_object_handle.downcast::<JsObject>() {
    Ok(x) => x,
    _ => JsObject::new(&mut ctx)
  };

  let rust_string: String = match js_object.get(&mut ctx, "myProp")?.downcast::<JsString>() {
    Ok(x) => x.value(),
    _ => String::new()
  };

  Ok(JsString::new(&mut ctx, rust_string))
}

register_module!(mut ctx, {
    cx.export_function("acceptsJsObjs", accepts_js_objs)
});
