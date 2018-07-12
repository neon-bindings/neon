#[macro_use]
extern crate neon;

use neon::vm::{Call, JsResult};
use neon::js::{Variant, JsString, JsObject};

// https://github.com/dfrankland/node-eyeliner/blob/master/native/src/js_object_utils.rs
pub fn js_value_to_string(js_string: Variant) -> String {
  match js_string {
    Variant::String(string) => string.value(),
    _ => panic!("heck"),
  }
}

fn accepts_js_objs(call: Call) -> JsResult<JsString> {
  let js_obj_handle = call.arguments.get(call.scope, 0).unwrap();
  let js_obj = js_obj_handle.downcast::<JsObject>().unwrap();

  let prop_js = js_obj.get(call.scope, "myProp").unwrap().variant();
  let prop_text: String = js_value_to_string(prop_js);

  Ok(JsString::new(call.scope, prop_text.as_str()).unwrap())
}

register_module!(m, {
  try!(m.export("accepts_js_objs", accepts_js_objs));
	Ok(())
});

// in lib/index.js
// const addon = require('../native');
// console.log(addon.accepts_js_objs({myProp: 'waddup neon'}));
