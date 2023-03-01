use neon::prelude::*;

use cargo_cp_artifact::cargo::Status;
use cargo_cp_artifact::cli;

fn run(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Skip the node binary name (argv[0]) and the script name (argv[1]).
    if let Status::Failure = cli::run(2) {
        cx.global()
            .get::<JsObject, _, _>(&mut cx, "process")?
            .get::<JsFunction, _, _>(&mut cx, "exit")?
            .call_with(&cx)
            .arg(cx.number(1))
            .exec(&mut cx)?;
    }

    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("run", run)?;
    Ok(())
}
