#[macro_use]
extern crate neon;

mod js {
  pub mod strings;
  pub mod integers;
  pub mod arrays;
  pub mod objects;
}

use js::strings::return_js_string;
use js::integers::return_js_integer;
use js::arrays::*;
use js::objects::*;

register_module!(m, {
    try!(m.export("return_js_string", return_js_string));

    try!(m.export("return_js_integer", return_js_integer));

    try!(m.export("return_js_array", return_js_array));
    try!(m.export("return_js_array_with_integer", return_js_array_with_integer));
    try!(m.export("return_js_array_with_string", return_js_array_with_string));

    try!(m.export("return_js_object", return_js_object));
    try!(m.export("return_js_object_with_integer", return_js_object_with_integer));
    try!(m.export("return_js_object_with_string", return_js_object_with_string));
    Ok(())
});


