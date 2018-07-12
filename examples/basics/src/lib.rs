#[macro_use]
extern crate neon;

// thanks to @rm-rf-etc for these: https://github.com/neon-bindings/neon/issues/260
use neon::vm::{Call, JsResult};
use neon::js::{
  Object, // <- https://github.com/neon-bindings/neon/issues/57
  JsFunction,
  JsObject,
  JsArray,
  JsBoolean,
  JsInteger,
  JsNull,
  JsNumber,
  JsString,
  JsUndefined};

/*
	// If you need to inspect the value of an expression:
	let () = JsBoolean::new();
*/

// Use _ to squelch warnings, or any name starting with _
fn get_null_sync(_: Call) -> JsResult<JsNull> {
  Ok(JsNull::new())
}

// Use _ to squelch warnings, or any name starting with _
fn get_undefined_sync(_: Call) -> JsResult<JsUndefined> {
  Ok(JsUndefined::new())
}

fn get_number_sync(call: Call) -> JsResult<JsNumber> {
  Ok(JsNumber::new(call.scope, 5f64))
}

fn get_integer_sync(call: Call) -> JsResult<JsInteger> {
  Ok(JsInteger::new(call.scope, 5i32))
}

fn get_string_sync(call: Call) -> JsResult<JsString> {
  Ok(JsString::new(call.scope, "my string").unwrap())
}

fn get_boolean_sync(call: Call) -> JsResult<JsBoolean> {
  Ok(JsBoolean::new(call.scope, false))
}

fn get_array_sync(call: Call) -> JsResult<JsArray> {
  let scope = call.scope;
  let array = JsArray::new(scope, 3);

  try!(array.set(0, JsInteger::new(scope, 1)));
  try!(array.set(1, JsInteger::new(scope, 2)));
  try!(array.set(2, JsInteger::new(scope, 3)));

  Ok(array)
}

fn get_object_sync(call: Call) -> JsResult<JsObject> {
  let scope = call.scope;
  let object = JsObject::new(scope);

  try!(object.set("prop1", JsInteger::new(scope, 1)));
  try!(object.set("prop2", JsInteger::new(scope, 2)));
  try!(object.set("prop3", JsInteger::new(scope, 3)));

  Ok(object)
}

fn get_function_sync(call: Call) -> JsResult<JsFunction> {
  fn func(call: Call) -> JsResult<JsInteger> {
    Ok(JsInteger::new(call.scope, 5))
  }
  Ok(JsFunction::new(call.scope, func).unwrap())
}

register_module!(m, {
	try!(m.export("getNullSync", get_null_sync));
	try!(m.export("getUndefinedSync", get_undefined_sync));
	try!(m.export("getNumberSync", get_number_sync));
	try!(m.export("getIntegerSync", get_integer_sync));
	try!(m.export("getStringSync", get_string_sync));
	try!(m.export("getBooleanSync", get_boolean_sync));
	try!(m.export("getArraySync", get_array_sync));
	try!(m.export("getObjectSync", get_object_sync));
	try!(m.export("getFunctionSync", get_function_sync));
	Ok(())
});

// in lib/index.js
//const addon = require('../native');
//
//console.log(`this is null: ${addon.getNullSync()}`);
//console.log(`this is undefined: ${addon.getUndefinedSync()}`);
//console.log(`this is pi: ${addon.getNumberSync()}`);
//console.log(`this is a 5: ${addon.getIntegerSync()}`);
//console.log(`this is a string: ${addon.getStringSync()}`);
//console.log(`this is false: ${addon.getBooleanSync()}`);
//console.log(`this is an array: ${addon.getArraySync()}`);
//console.log(`this is an object: ${JSON.stringify(addon.getObjectSync())}`);
//const returnFive = addon.getFunctionSync()
//console.log(returnFive(), returnFive(), returnFive())

