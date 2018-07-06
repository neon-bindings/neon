use neon::js::{JsString, JsNumber, Borrow, JsError, ErrorKind};
use neon::vm::{Context, Handle};

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
    init(mut cx) {
      let id = cx.argument::<JsNumber>(0)?;
      let first_name: Handle<JsString> = cx.argument::<JsString>(1)?;
      let last_name: Handle<JsString> = cx.argument::<JsString>(2)?;
      let email: Handle<JsString> = cx.argument::<JsString>(3)?;

      Ok(User {
        id: id.value() as i32,
        first_name: first_name.value(),
        last_name: last_name.value(),
        email: email.value(),
      })
    }

    method get(mut cx) {
      let attr: String = cx.argument::<JsString>(0)?.value();

      let this = cx.this();

      match &attr[..] {
        "id" => {
          let id = {
            let guard = cx.lock();
            let user = this.borrow(&guard);
            user.id
          };
          Ok(cx.number(id).upcast())
        },
        "first_name" => {
          let first_name = {
            let guard = cx.lock();
            let user = this.borrow(&guard);
            user.first_name.clone()
          };
          Ok(cx.string(&first_name).upcast())
        },
        "last_name" => {
          let last_name = {
            let guard = cx.lock();
            let user = this.borrow(&guard);
            user.last_name.clone()
          };
          Ok(cx.string(&last_name).upcast())
        },
        "email" => {
          let email = {
            let guard = cx.lock();
            let user = this.borrow(&guard);
            user.email.clone()
          };
          Ok(cx.string(&email).upcast())
        },
        _ => JsError::throw(&mut cx, ErrorKind::TypeError, "property does not exist")
      }
    }

    method panic(_) {
      panic!("User.prototype.panic")
    }
  }
}
