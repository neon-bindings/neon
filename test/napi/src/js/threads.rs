use std::{cell::RefCell, sync::Arc, time::Duration};

use neon::{prelude::*, types::buffer::TypedArray};

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
        channel.send(move |mut cx| callback.into_inner(&mut cx).call_with(&cx).exec(&mut cx))
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
                callback
                    .into_inner(&mut cx)
                    .call_with(&cx)
                    .arg(cx.number(i as f64))
                    .exec(&mut cx)
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
                callback
                    .into_inner(&mut cx)
                    .call_with(&cx)
                    .arg(cx.string(greeting))
                    .exec(&mut cx)
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
            let _ = shutdown.into_inner(cx).call_with(cx).exec(cx);
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
                    callback
                        .into_inner(&mut cx)
                        .call_with(&cx)
                        .arg(cx.undefined())
                        .exec(&mut cx)
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

pub fn channel_join(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Function to fetch a message for processing
    let get_message = cx.argument::<JsFunction>(0)?.root(&mut cx);
    // Callback into JavaScript with completion
    let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let channel = cx.channel();

    // Spawn a Rust thread to stop blocking the event loop
    std::thread::spawn(move || {
        // Give a chance for the data to change
        std::thread::sleep(Duration::from_millis(100));

        // Get the current message
        let message = channel
            .send(move |mut cx| {
                let this = cx.undefined();

                get_message
                    .into_inner(&mut cx)
                    .call(&mut cx, this, [])?
                    .downcast_or_throw::<JsString, _>(&mut cx)
                    .map(|v| v.value(&mut cx))
            })
            .join()
            .unwrap();

        // Process the message
        let response = format!("Received: {}", message);

        // Call back to JavaScript with the response
        channel.send(move |mut cx| {
            let this = cx.undefined();
            let args = [cx.string(response).upcast()];

            callback.into_inner(&mut cx).call(&mut cx, this, args)?;

            Ok(())
        });
    });

    Ok(cx.undefined())
}

pub fn sum(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let nums = cx.argument::<JsTypedArray<f64>>(0)?.as_slice(&cx).to_vec();

    let promise = cx
        .task(move || nums.into_iter().sum())
        .promise(|mut cx, n: f64| Ok(cx.number(n)));

    Ok(promise)
}

pub fn sum_manual_promise(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let nums = cx.argument::<JsTypedArray<f64>>(0)?.as_slice(&cx).to_vec();

    let (deferred, promise) = cx.promise();

    cx.task(move || nums.into_iter().sum())
        .and_then(move |mut cx, n: f64| {
            let n = cx.number(n);
            deferred.resolve(&mut cx, n);
            Ok(())
        });

    Ok(promise)
}

pub fn sum_rust_thread(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let nums = cx.argument::<JsTypedArray<f64>>(0)?.as_slice(&cx).to_vec();

    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    std::thread::spawn(move || {
        let n: f64 = nums.into_iter().sum();

        deferred.settle_with(&channel, move |mut cx| Ok(cx.number(n)));
    });

    Ok(promise)
}

pub fn leak_promise(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let (_, promise) = cx.promise();

    Ok(promise)
}

pub fn channel_panic(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let channel = cx.channel();

    std::thread::spawn(move || {
        channel.send(move |_| -> NeonResult<()> {
            panic!("{}", msg);
        })
    });

    Ok(cx.undefined())
}

pub fn channel_throw(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let channel = cx.channel();

    std::thread::spawn(move || {
        channel.send(move |mut cx| {
            cx.throw_error(msg)?;
            Ok(())
        })
    });

    Ok(cx.undefined())
}

pub fn channel_panic_throw(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let channel = cx.channel();

    std::thread::spawn(move || {
        channel.send(move |mut cx| {
            // Throw an exception, but ignore the `Err(Throw)`
            let _ = cx.throw_error::<_, ()>(msg);
            // Attempting to throw another error while already throwing should `panic`
            cx.throw_error("Unreachable")?;

            Ok(())
        })
    });

    Ok(cx.undefined())
}

struct CustomPanic(String);

pub fn channel_custom_panic(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let channel = cx.channel();

    std::thread::spawn(move || {
        channel.send(move |_| -> NeonResult<()> {
            std::panic::panic_any(CustomPanic(msg));
        })
    });

    Ok(cx.undefined())
}

pub fn custom_panic_downcast(mut cx: FunctionContext) -> JsResult<JsString> {
    let panic = cx.argument::<JsBox<CustomPanic>>(0)?;

    Ok(cx.string(&panic.0))
}

pub fn task_panic_execute(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);

    cx.task(move || panic!("{}", msg)).and_then(|_, _| Ok(()));

    Ok(cx.undefined())
}

pub fn task_panic_complete(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);

    cx.task(|| {}).and_then(move |_, _| panic!("{}", msg));

    Ok(cx.undefined())
}

pub fn task_throw(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);

    cx.task(|| {}).and_then(move |mut cx, _| {
        cx.throw_error(msg)?;
        Ok(())
    });

    Ok(cx.undefined())
}

pub fn task_panic_throw(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);

    cx.task(|| {}).and_then(move |mut cx, _| {
        // Throw an exception, but ignore the `Err(Throw)`
        let _ = cx.throw_error::<_, ()>(msg);
        // Attempting to throw another error while already throwing should `panic`
        cx.throw_error("Unreachable")?;

        Ok(())
    });

    Ok(cx.undefined())
}

pub fn task_custom_panic(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);

    cx.task(move || std::panic::panic_any(CustomPanic(msg)))
        .and_then(|_, _| Ok(()));

    Ok(cx.undefined())
}

pub fn task_reject_promise(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let promise = cx
        .task(move || {})
        .promise::<JsValue, _>(move |mut cx, _| cx.throw_error(msg));

    Ok(promise)
}

pub fn task_panic_execute_promise(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let promise = cx
        .task(move || panic!("{}", msg))
        .promise(|mut cx, _| Ok(cx.undefined()));

    Ok(promise)
}

pub fn task_panic_complete_promise(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let promise = cx
        .task(|| ())
        .promise::<JsValue, _>(move |_, _| panic!("{}", msg));

    Ok(promise)
}

pub fn task_panic_throw_promise(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let promise = cx.task(|| ()).promise(move |mut cx, _| {
        // Throw an exception, but ignore the `Err(Throw)`
        let _ = cx.throw_error::<_, ()>(msg);
        // Attempting to throw another error while already throwing should `panic`
        cx.throw_error("Unreachable")?;

        Ok(cx.undefined())
    });

    Ok(promise)
}

pub fn deferred_settle_with_throw(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let (deferred, promise) = cx.promise();
    let channel = cx.channel();

    std::thread::spawn(move || {
        deferred.try_settle_with(&channel, move |mut cx| {
            cx.throw_error(msg)?;

            Ok(cx.undefined())
        })
    });

    Ok(promise)
}

pub fn deferred_settle_with_panic(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let (deferred, promise) = cx.promise();
    let channel = cx.channel();

    std::thread::spawn(move || {
        deferred.try_settle_with::<JsValue, _>(&channel, move |_| {
            panic!("{}", msg);
        })
    });

    Ok(promise)
}

pub fn deferred_settle_with_panic_throw(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let msg = cx.argument::<JsString>(0)?.value(&mut cx);
    let (deferred, promise) = cx.promise();
    let channel = cx.channel();

    std::thread::spawn(move || {
        deferred.try_settle_with(&channel, move |mut cx| {
            // Throw an exception, but ignore the `Err(Throw)`
            let _ = cx.throw_error::<_, ()>(msg);
            // Attempting to throw another error while already throwing should `panic`
            cx.throw_error("Unreachable")?;

            Ok(cx.undefined())
        })
    });

    Ok(promise)
}
