#[macro_use]
extern crate neon;

use neon::prelude::*;
// thanks to @rm-rf-etc for these: https://github.com/neon-bindings/neon/issues/260

// Use _ to squelch warnings, or any name starting with _
fn get_null_sync(_: FunctionContext) -> JsResult<JsNull> {
  Ok(JsNull::new())
}

// Use _ to squelch warnings, or any name starting with _
fn get_undefined_sync(_: FunctionContext) -> JsResult<JsUndefined> {
  Ok(JsUndefined::new())
}

fn get_number_sync(mut ctx: FunctionContext) -> JsResult<JsNumber> {
  Ok(ctx.number(3.14f64))
}

fn get_string_sync(mut ctx: FunctionContext) -> JsResult<JsString> {
  Ok(ctx.string("my string"))
}

fn get_boolean_sync(mut ctx: FunctionContext) -> JsResult<JsBoolean> {
  Ok(ctx.boolean(false))
}

fn get_array_sync(mut ctx: FunctionContext) -> JsResult<JsArray> {
  let array = ctx.empty_array();
  let val1 = ctx.number(1);
  let val2 = ctx.number(2);
  let val3 = ctx.number(3);
  array.set(&mut ctx, 0, val1)?;
  array.set(&mut ctx, 1, val2)?;
  array.set(&mut ctx, 2, val3)?;

  Ok(array)
}

fn get_object_sync(mut ctx: FunctionContext) -> JsResult<JsObject> {
  let object = ctx.empty_object();
  let prop1 = ctx.number(1);
  let prop2 = ctx.number(2);
  let prop3 = ctx.number(3);

  object.set(&mut ctx, "prop1", prop1)?;
  object.set(&mut ctx, "prop2", prop2)?;
  object.set(&mut ctx, "prop3", prop3)?;

  Ok(object)
}

fn get_function_sync(mut ctx: FunctionContext) -> JsResult<JsFunction> {
  fn func(mut ctx: FunctionContext) -> JsResult<JsNumber> {
    Ok(ctx.number(5))
  }
  JsFunction::new(&mut ctx, func)
}

register_module!(mut ctx, {
	ctx.export_function("getNullSync", get_null_sync);
	ctx.export_function("getUndefinedSync", get_undefined_sync);
	ctx.export_function("getNumberSync", get_number_sync);
	ctx.export_function("getIntegerSync", get_integer_sync);
	ctx.export_function("getStringSync", get_string_sync);
	ctx.export_function("getBooleanSync", get_boolean_sync);
	ctx.export_function("getArraySync", get_array_sync);
	ctx.export_function("getObjectSync", get_object_sync);
	ctx.export_function("getFunctionSync", get_function_sync);
	Ok(())
});
