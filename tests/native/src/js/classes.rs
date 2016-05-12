use neon::js::{JsString, JsNumber, JsValue, JsObject, JsFunction, Object};
use neon::vm::{Call, JsResult};
use neon::js::class::{Class, JsClass};
use neon::mem::Handle;

pub struct User {
  id: i32,
  first_name: String,
  last_name: String,
  email: String,
}

declare_types! {
  pub class JsUser for User {
    init(call) {
      let scope = call.scope;
      let id = try!(try!(call.arguments.require(scope, 0)).check::<JsNumber>());
      let first_name: Handle<JsString> = try!(try!(call.arguments.require(scope, 1)).check::<JsString>());
      let last_name: Handle<JsString> = try!(try!(call.arguments.require(scope, 2)).check::<JsString>());
      let email: Handle<JsString> = try!(try!(call.arguments.require(scope, 3)).check::<JsString>());

      Ok(User {
        id: id.value() as i32,
        first_name: first_name.value(),
        last_name: last_name.value(),
        email: email.value(),
      })
    }
  }
}

pub fn return_js_class(call: Call) -> JsResult<JsUser> {
    let scope = call.scope;
    let class: Handle<JsClass<JsUser>> = try!(JsUser::class(scope));
    let ctor: Handle<JsFunction<JsUser>> = try!(class.constructor(scope));
    let args: Vec<Handle<JsValue>> = vec![];
    ctor.construct(scope, args)
}
