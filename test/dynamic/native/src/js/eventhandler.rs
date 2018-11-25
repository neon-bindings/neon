use neon::prelude::*;

use std::thread;
use std::sync::mpsc;
use std::time::Duration;

pub struct Emitter {
  cb: Option<EventHandler>,
}

declare_types! {
  pub class JsEmitter for Emitter {
    init(_) {
      Ok(Emitter {
        cb: None
      })
    }

    constructor(mut cx) {
      let mut this = cx.this();
      let f = this.get(&mut cx, "emit")?.downcast::<JsFunction>().or_throw(&mut cx)?;
      let cb = EventHandler::bind(this, f);
      {
        let guard = cx.lock();
        let mut callback = this.borrow_mut(&guard);
        callback.cb = Some(cb);
      }
      Ok(None)
    }

    method start(mut cx) {
        let this = cx.this();
        let cb = {
            let guard = cx.lock();
            let callback = this.borrow(&guard);
            callback.cb.clone()
        };
        if let Some(cb) = cb {
          let (sender, receiver) = mpsc::channel();
            for i in 0..10 {
              let cb = cb.clone();
              let sender = sender.clone();
              thread::spawn(move || {
                // do some work ....
                thread::sleep(Duration::from_millis(40));
                cb.schedule(move |cx| {
                  let args : Vec<Handle<JsValue>> = vec![cx.string("progress").upcast(), cx.number(i).upcast()];
                  args
                });
                // ignore failure, test will fail if not called
                let _r = sender.send(10);
              });
            }
            thread::spawn(move || {
              let mut sum = 0;
              for _i in 0..10 {
                sum += receiver.recv().unwrap_or(0);
              }
              cb.schedule(move |cx| {
                let args : Vec<Handle<JsValue>> = vec![cx.string("end").upcast(), cx.number(sum).upcast()];
                args
              });
            });
        }
        Ok(cx.undefined().upcast())
    }

    method shutdown(mut cx) {
      let mut this = cx.this();
      {
        let guard = cx.lock();
        let mut callback = this.borrow_mut(&guard);
        callback.cb = None;
      }
      Ok(cx.undefined().upcast())
    }
  }
}

pub struct TestEmitter {
  cb: Option<EventHandler>,
}

declare_types! {
  pub class JsTestEmitter for TestEmitter {
    init(_) {
      Ok(TestEmitter {
        cb: None
      })
    }

    constructor(mut cx) {
      let mut this = cx.this();
      let f = cx.argument::<JsFunction>(0)?;
      let cb = EventHandler::bind(this, f);
      {
        let guard = cx.lock();
        let mut callback = this.borrow_mut(&guard);
        callback.cb = Some(cb);
      }
      Ok(None)
    }

    method start(mut cx) {
      let this = cx.this();
      let cb = {
          let guard = cx.lock();
          let callback = this.borrow(&guard);
          callback.cb.clone()
      };
      if let Some(cb) = cb {
        thread::spawn(move || {
          cb.schedule_with(move |cx, this, callback| {
            let args : Vec<Handle<JsValue>> = vec![cx.string("number").upcast()];
            let result = callback.call(cx, this, args);
            let cmd = match result {
              Ok(v) => {
                if let Ok(number) = v.downcast::<JsNumber>() {
                   if number.value() == 12f64 {
                     "done".into()
                   } else {
                     "wrong number".into()
                   }
                } else {
                  "no number returned".into()
                }
              },
              Err(e) => format!("threw {}", e)
            };
            let args : Vec<Handle<JsValue>> = vec![cx.string(cmd).upcast()];
            let _result = callback.call(cx, this, args);
          });
        });
      }
      Ok(cx.undefined().upcast())
    }

    method shutdown(mut cx) {
      let mut this = cx.this();
      {
        let guard = cx.lock();
        let mut callback = this.borrow_mut(&guard);
        callback.cb = None;
      }
      Ok(cx.undefined().upcast())
    }
  }
}
