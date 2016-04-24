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
    m.export("return_js_string", return_js_string);

    m.export("return_js_integer", return_js_integer);

    m.export("return_js_array", return_js_array);
    m.export("return_js_array_with_integer", return_js_array_with_integer);
    m.export("return_js_array_with_string", return_js_array_with_string);

    m.export("return_js_object", return_js_object);
    m.export("return_js_object_with_integer", return_js_object_with_integer);
    m.export("return_js_object_with_string", return_js_object_with_string)
});


