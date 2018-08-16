#[macro_use]
extern crate neon;

use neon::prelude::*;

fn accepts_js_arrays(mut ctx: FunctionContext) -> JsResult<JsNumber> {
  let js_arr_handle: Handle<JsArray> = ctx.argument(0)?;

  let vec: Vec<Handle<JsValue>> = js_arr_handle.to_vec(&mut ctx)?;
  let vec_of_numbers: Vec<f64> = vec.iter().map(|js_value| {
    js_value.downcast::<JsNumber>().unwrap_or(ctx.number(0)).value()
  }).collect();
  let sum: f64 = vec_of_numbers.iter().sum();
  Ok(JsNumber::new(&mut ctx, sum))
}

register_module!(mut ctx, {
    ctx.export_function("acceptsJsArrays", accepts_js_arrays)
});
