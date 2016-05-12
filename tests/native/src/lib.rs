#[macro_use]
extern crate neon;

mod js {
    pub mod strings;
    pub mod numbers;
    pub mod arrays;
    pub mod objects;
    pub mod functions;
    pub mod classes;
}

use js::strings::return_js_string;
use js::numbers::*;
use js::arrays::*;
use js::objects::*;
use js::functions::*;
use js::classes::*;

register_module!(m, {
    try!(m.export("return_js_string", return_js_string));

    try!(m.export("return_js_number", return_js_number));
    try!(m.export("return_large_js_number", return_large_js_number));
    try!(m.export("return_negative_js_number", return_negative_js_number));
    try!(m.export("return_float_js_number", return_float_js_number));
    try!(m.export("return_negative_float_js_number", return_negative_float_js_number));
    try!(m.export("accept_and_return_js_number", accept_and_return_js_number));
    try!(m.export("accept_and_return_large_js_number", accept_and_return_large_js_number));
    try!(m.export("accept_and_return_float_js_number", accept_and_return_float_js_number));
    try!(m.export("accept_and_return_negative_js_number", accept_and_return_negative_js_number));

    try!(m.export("return_js_array", return_js_array));
    try!(m.export("return_js_array_with_integer", return_js_array_with_integer));
    try!(m.export("return_js_array_with_string", return_js_array_with_string));

    try!(m.export("return_js_object", return_js_object));
    try!(m.export("return_js_object_with_integer", return_js_object_with_integer));
    try!(m.export("return_js_object_with_string", return_js_object_with_string));

    try!(m.export("return_js_function", return_js_function));
    try!(m.export("call_js_function", call_js_function));
    try!(m.export("construct_js_function", construct_js_function));

    try!(m.export("return_js_class", return_js_class));

    // try!(m.export("User", JsUser::new));

    Ok(())
});
