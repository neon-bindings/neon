use serde::Deserialize;
use std::{fmt, fs, io, path, time::Duration};

use criterion::Criterion;
use neon::prelude::*;

mod pokemon;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchOptions {
    report_file: Option<path::PathBuf>,
    #[serde(default)]
    neon: bool,
    #[serde(default)]
    json: bool,
}

fn serialize(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let options = cx
        .argument_opt(0)
        .unwrap_or_else(|| cx.empty_object().upcast());

    let options = neon::deserialize::<BenchOptions, _, _>(&mut cx, options)?;
    let mut c = Criterion::default();
    let mut group = c.benchmark_group("Serialize");
    let pokedex = pokemon::pokedex();
    let this = cx.undefined();

    group.measurement_time(Duration::from_secs(10));

    // Fill the cache to avoid including it in profiles
    neon::serialize::<JsValue, _, _>(&mut cx, pokedex)?;

    // Note: We create functions because a mix of criterion and release
    // optimizations is somehow causing a bug where handle scopes from
    // `execute_scoped` fail to drop.
    let neon = JsFunction::new(&mut cx, move |mut cx| {
        neon::serialize::<JsValue, _, _>(&mut cx, pokedex)?;
        Ok(cx.undefined())
    })?;

    let parse = cx
        .global()
        .get::<JsObject, _, _>(&mut cx, "JSON")?
        .get::<JsFunction, _, _>(&mut cx, "parse")?
        .root(&mut cx);

    let json = JsFunction::new(&mut cx, move |mut cx| {
        let parse = parse.to_inner(&mut cx);
        let s = serde_json::to_string(pokedex).or_throw(&mut cx)?;
        let s = cx.string(s).upcast();
        let this = cx.undefined();
        parse.call(&mut cx, this, [s])?;
        Ok(cx.undefined())
    })?;

    let guard = options
        .report_file
        .is_some()
        .then(|| pprof::ProfilerGuardBuilder::default().build())
        .transpose()
        .or_else(|err| cx.throw_error(err.to_string()))?;

    if options.neon {
        group.bench_function("neon::serialize", |b| {
            b.iter(|| neon.call(&mut cx, this, &[]))
        });
    }

    if options.json {
        group.bench_function("serde_json", |b| {
            b.iter(|| json.call(&mut cx, this, &[]))
        });
    }

    group.finish();

    if let (Some(guard), Some(report_file)) = (guard, options.report_file.as_ref()) {
        let report = guard.report().build().or_throw(&mut cx)?;
        let profile = report.pprof().or_throw(&mut cx)?;
        let mut file = fs::File::create(report_file).or_throw(&mut cx)?;

        let mut content = Vec::new();

        pprof::protos::Message::encode(&profile, &mut content).or_throw(&mut cx)?;
        io::Write::write_all(&mut file, &content).or_throw(&mut cx)?;
    }

    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("serialize", serialize)?;

    Ok(())
}

trait ResultExt<T, E> {
    fn or_throw<'cx, C>(self, cx: &mut C) -> NeonResult<T>
    where
        C: Context<'cx>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: fmt::Display,
{
    fn or_throw<'cx, C>(self, cx: &mut C) -> NeonResult<T>
    where
        C: Context<'cx>,
    {
        match self {
            Ok(v) => Ok(v),
            Err(e) => cx.throw_error(e.to_string()),
        }
    }
}
