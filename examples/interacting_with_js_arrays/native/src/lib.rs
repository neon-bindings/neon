#[macro_use]
extern crate neon;

use neon::vm::{Call, JsResult};
use neon::js::{Variant, JsNumber, JsArray};

fn accepts_js_arrays(call: Call) -> JsResult<JsNumber> {
  let js_arr_handle = call.arguments.get(call.scope, 0)?;
  let vec: Vec<_> = js_arr_handle.check::<JsArray>()?.to_vec(call.scope)?;

  let vec_of_numbers: Vec<_> = vec.iter().map(|js_value| {
    let js_number = js_value::check::<JsNumber>()?;
    Ok(js_number.value())
  }).collect()?;
  let sum = vec_of_numbers.iter().sum();
  Ok(JsNumber::new(call.scope, sum))
}

register_module!(m, {
  m.export("accepts_js_arrays", accepts_js_arrays)?;
	Ok(())
});
