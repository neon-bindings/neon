use neon::prelude::*;

use once_cell::sync::OnceCell;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn get_num(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let callback = cx.argument::<JsFunction>(0)?;
    let this = cx.undefined();
    let args = Vec::<Handle<JsValue>>::with_capacity(0);
    let promise = callback.call(&mut cx, this, args)?;
    let future = promise.to_future_adapter(&mut cx, |mut cx, v| {
        v.unwrap_or_else(|_| panic!("Promise rejected"))
            .downcast::<JsNumber, _>(&mut cx)
            .unwrap()
            .value(&mut cx)
    });

    RUNTIME.get().unwrap().spawn(async {
        let n = future.await;

        println!("n is {}", n);
    });

    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    let _ = RUNTIME.get_or_init(|| Runtime::new().unwrap());

    cx.export_function("getNum", get_num)?;

    Ok(())
}
