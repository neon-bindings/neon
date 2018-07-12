#[macro_use]
extern crate neon;

use neon::vm::{Call, JsResult};
use neon::js::{Variant, JsNumber, JsArray};

// https://github.com/dfrankland/node-eyeliner/blob/master/native/src/js_object_utils.rs
pub fn js_value_to_float(js_number: Variant) -> f64 {
  match js_number {
    Variant::Number(number) => number.value(),
    _ => panic!("heck"),
  }
}

fn accepts_js_arrays(call: Call) -> JsResult<JsNumber> {
  let js_arr_handle = call.arguments.get(call.scope, 0).unwrap();
  let js_arr = js_arr_handle.downcast::<JsArray>().unwrap().to_vec(call.scope)?;

  let vec_of_numbers: Vec<_> = js_arr.iter().map(move|js_value| {
    let js_number = js_value.variant();
    js_value_to_float(js_number)
  }).collect();
  let sum = vec_of_numbers.iter().sum();
  Ok(JsNumber::new(call.scope, sum))
}

register_module!(m, {
  try!(m.export("accepts_js_arrays", accepts_js_arrays));
	Ok(())
});

// in lib/index.js
//const addon = require('../native');
//console.log(addon.accepts_js_arrays([0.1, 1.2, 2.3]));