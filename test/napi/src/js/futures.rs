use {
    neon::{prelude::*, types::buffer::TypedArray},
    once_cell::sync::OnceCell,
    tokio::runtime::Runtime,
};

fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME
        .get_or_try_init(Runtime::new)
        .or_else(|err| cx.throw_error(&err.to_string()))
}

// Accepts two functions that take no parameters and return numbers.
// Resolves with the sum of the two numbers.
// Purpose: Test the `Future` implementation on `JoinHandle`
pub fn lazy_async_add(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let get_x = cx.argument::<JsFunction>(0)?.root(&mut cx);
    let get_y = cx.argument::<JsFunction>(1)?.root(&mut cx);
    let channel = cx.channel();
    let runtime = runtime(&mut cx)?;
    let (deferred, promise) = cx.promise();

    runtime.spawn(async move {
        let result = channel
            .send(move |mut cx| {
                let get_x = get_x.into_inner(&mut cx);
                let get_y = get_y.into_inner(&mut cx);

                let x: Handle<JsNumber> = get_x.call_with(&cx).apply(&mut cx)?;
                let y: Handle<JsNumber> = get_y.call_with(&cx).apply(&mut cx)?;

                Ok((x.value(&mut cx), y.value(&mut cx)))
            })
            .await
            .map(|(x, y)| x + y);

        deferred.settle_with(&channel, move |mut cx| {
            let result = result.or_throw(&mut cx)?;

            Ok(cx.number(result))
        });
    });

    Ok(promise)
}

// Accepts a function that returns a `Promise<Float64Array>`.
// Resolves with the sum of all numbers.
// Purpose: Test `JsPromise::to_future`.
pub fn lazy_async_sum(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let nums = cx
        .argument::<JsFunction>(0)?
        .call_with(&cx)
        .apply::<JsPromise, _>(&mut cx)?
        .to_future(&mut cx, |mut cx, nums| {
            let nums = nums
                .or_throw(&mut cx)?
                .downcast_or_throw::<JsTypedArray<f64>, _>(&mut cx)?
                .as_slice(&cx)
                .to_vec();

            Ok(nums)
        })?;

    let (deferred, promise) = cx.promise();
    let channel = cx.channel();
    let runtime = runtime(&mut cx)?;

    runtime.spawn(async move {
        let result = nums.await.map(|nums| nums.into_iter().sum::<f64>());

        deferred.settle_with(&channel, move |mut cx| {
            let result = result.or_throw(&mut cx)?;

            Ok(cx.number(result))
        });
    });

    Ok(promise)
}
