use neon::prelude::*;
use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn runtime<'cx, C: Context<'cx>>(cx: &mut C) -> NeonResult<&Runtime> {
    if let Some(runtime) = RUNTIME.get() {
        Ok(runtime)
    } else {
        cx.throw_error("Runtime is not initialized")
    }
}

async fn add(a: f64, b: f64) -> f64 {
    a + b
}

fn js_add(mut cx: FunctionContext) -> JsResult<JsPromise> {
    let a = cx.argument::<JsNumber>(0)?.value(&mut cx);
    let b = cx.argument::<JsPromise>(1)?.root(&mut cx);
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    runtime(&mut cx)?.spawn(async move {
        let res = channel
            .send(move |mut cx| {
                b.into_inner(&mut cx).to_future(&mut cx, |cx, res| {
                    Ok(res
                        .or_throw(cx)?
                        .downcast_or_throw::<JsNumber, _>(cx)?
                        .value(cx))
                })
            })
            .await;

        let res = match res {
            Ok(v) => Ok(v.await),
            Err(err) => Err(err),
        };

        let res = match res {
            Ok(Ok(b)) => Ok(add(a, b).await),
            Ok(Err(err)) => Err(err),
            Err(err) => Err(err),
        };

        deferred.settle_with(&channel, move |mut cx| {
            res.map(|n| cx.number(n))
                .or_else(|err| cx.throw_error(err.to_string()))
        });
    });

    Ok(promise)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    let runtime = Runtime::new().or_else(|err| cx.throw_error(err.to_string()))?;
    let _ = RUNTIME.set(runtime);

    cx.export_function("add", js_add)?;
    cx.export_function("test", |mut cx| {
        cx.channel().send::<(), _>(|_| panic!("Oh, no!"));
        Ok(cx.undefined())
    })?;

    Ok(())
}
