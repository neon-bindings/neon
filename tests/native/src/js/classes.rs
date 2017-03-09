use neon::js::{JsString, JsNumber};
use neon::mem::Handle;
use neon::vm::Lock;
use neon::js::error::{JsError, Kind};

pub struct User {
  id: i32,
  first_name: String,
  last_name: String,
  email: String,
}

type Unit = ();

declare_types! {
  pub class JsPanickyAllocator for Unit {
    init(_) {
      panic!("allocator panicking")
    }
  }

  pub class JsPanickyConstructor for Unit {
    init(_) {
      Ok(())
    }

    call(_) {
      panic!("constructor call panicking")
    }

    constructor(_) {
      panic!("constructor panicking")
    }
  }

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

    method get(call) {
      let scope = call.scope;

      let attr: String = try!(try!(call.arguments.require(scope, 0)).check::<JsString>()).value();

      match &attr[..] {
        "id" => {
          let id = call.arguments.this(scope).grab(|user| { user.id.clone() });
          Ok(JsNumber::new(scope, id as f64).upcast())
        },
        "first_name" => {
          let first_name = call.arguments.this(scope).grab(|user| { user.first_name.clone() });
          Ok(try!(JsString::new_or_throw(scope, &first_name[..])).upcast())
        },
        "last_name" => {
          let last_name = call.arguments.this(scope).grab(|user| { user.last_name.clone() });
          Ok(try!(JsString::new_or_throw(scope, &last_name[..])).upcast())
        },
        "email" => {
          let email = call.arguments.this(scope).grab(|user| { user.email.clone() });
          Ok(try!(JsString::new_or_throw(scope, &email[..])).upcast())
        },
        _ => JsError::throw(Kind::TypeError, "property does not exist")
      }
    }

    method panic(_) {
      panic!("User.prototype.panic")
    }
  }
}
