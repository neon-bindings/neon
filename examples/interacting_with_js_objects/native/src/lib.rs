#[macro_use]
extern crate neon;

use neon::vm::{Call, JsResult};
use neon::js::{Variant, JsString, JsObject};

fn accepts_js_objs(call: Call) -> JsResult<JsString> {
  let js_obj_handle = call.arguments.get(call.scope, 0)?;
  let js_obj = js_obj_handle.check::<JsObject>()?;

  let prop_js = js_obj.get(call.scope, "myProp")?;
  let prop_text: String = prop_js::check::<JsString>()?.value();

  JsString::new(call.scope, prop_text.as_str())
}

register_module!(m, {
  m.export("accepts_js_objs", accepts_js_objs)?;
	Ok(())
});
