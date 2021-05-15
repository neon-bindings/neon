use std::cell::RefCell;
use std::sync::Arc;

use neon::prelude::*;

pub fn useless_root(mut cx: FunctionContext) -> JsResult<JsObject> {
    let object = cx.argument::<JsObject>(0)?;
    let root = object.root(&mut cx);
    let object = root.into_inner(&mut cx);

    Ok(object)
}

pub fn thread_callback(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let channel = cx.channel();

    std::thread::spawn(move || {
        channel.send(move |mut cx| {
            let callback = callback.into_inner(&mut cx);
            let this = cx.undefined();
            let args = Vec::<Handle<JsValue>>::new();

            callback.call(&mut cx, this, args)?;

            Ok(())
        })
    });

    Ok(cx.undefined())
}

pub fn multi_threaded_callback(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let n = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let channel = Arc::new(cx.channel());

    for i in 0..(n as usize) {
        let callback = callback.clone(&mut cx);
        let channel = Arc::clone(&channel);

        std::thread::spawn(move || {
            channel.send(move |mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                let args = vec![cx.number(i as f64)];

                callback.call(&mut cx, this, args)?;

                Ok(())
            })
        });
    }

    callback.drop(&mut cx);

    Ok(cx.undefined())
}

type BoxedGreeter = JsBox<RefCell<AsyncGreeter>>;

pub struct AsyncGreeter {
    greeting: String,
    callback: Root<JsFunction>,
    shutdown: Option<Root<JsFunction>>,
    channel: Arc<Channel>,
}

impl AsyncGreeter {
    fn greet<'a, C: Context<'a>>(&self, mut cx: C) -> JsResult<'a, JsUndefined> {
        let greeting = self.greeting.clone();
        let callback = self.callback.clone(&mut cx);
        let channel = Arc::clone(&self.channel);

        std::thread::spawn(move || {
            channel.send(|mut cx| {
                let callback = callback.into_inner(&mut cx);
                let this = cx.undefined();
                let args = vec![cx.string(greeting)];

                callback.call(&mut cx, this, args)?;

                Ok(())
            })
        });

        Ok(cx.undefined())
    }
}

impl Finalize for AsyncGreeter {
    fn finalize<'a, C: Context<'a>>(self, cx: &mut C) {
        let Self {
            callback, shutdown, ..
        } = self;

        if let Some(shutdown) = shutdown {
            let shutdown = shutdown.into_inner(cx);
            let this = cx.undefined();
            let args = Vec::<Handle<JsValue>>::new();
            let _ = shutdown.call(cx, this, args);
        }

        callback.drop(cx);
    }
}

pub fn greeter_new(mut cx: FunctionContext) -> JsResult<BoxedGreeter> {
    let greeting = cx.argument::<JsString>(0)?.value(&mut cx);
    let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let shutdown = cx.argument_opt(2);

    let channel = cx.channel();
    let shutdown = shutdown
        .map(|v| v.downcast_or_throw::<JsFunction, _>(&mut cx))
        .transpose()?
        .map(|v| v.root(&mut cx));

    let greeter = cx.boxed(RefCell::new(AsyncGreeter {
        greeting,
        callback,
        shutdown,
        channel: Arc::new(channel),
    }));

    Ok(greeter)
}

pub fn greeter_greet(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let greeter = cx.argument::<BoxedGreeter>(0)?;
    let greeter = greeter.borrow();

    greeter.greet(cx)
}

pub fn leak_channel(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let channel = Box::new({
        let mut channel = cx.channel();
        channel.unref(&mut cx);
        channel
    });

    Box::leak(channel);

    Ok(cx.undefined())
}

pub fn drop_global_queue(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    struct Wrapper {
        callback: Option<Root<JsFunction>>,
        channel: Channel,
    }

    impl Finalize for Wrapper {}

    // To verify that the type is dropped on the global drop queue, the callback
    // is called from the `Drop` impl on `Wrapper`
    impl Drop for Wrapper {
        fn drop(&mut self) {
            if let Some(callback) = self.callback.take() {
                self.channel.send(|mut cx| {
                    let callback = callback.into_inner(&mut cx);
                    let this = cx.undefined();
                    let args = vec![cx.undefined()];

                    callback.call(&mut cx, this, args)?;

                    Ok(())
                });
            }
        }
    }

    let callback = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let channel = cx.channel();

    let wrapper = cx.boxed(Wrapper {
        callback: Some(callback),
        channel,
    });

    // Put the `Wrapper` instance in a `Root` and drop it
    // Without the global drop queue, this will panic
    let _ = wrapper.root(&mut cx);

    Ok(cx.undefined())
}
