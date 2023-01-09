use serde::Deserialize;
use std::{fmt, time::Duration};

use criterion::Criterion;
use neon::prelude::*;

#[cfg(not(windows))]
use std::{fs, io, path};

mod pokemon;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchOptions {
    #[cfg(not(windows))]
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

    group.measurement_time(Duration::from_secs(10));

    let parse = cx
        .global()
        .get::<JsObject, _, _>(&mut cx, "JSON")?
        .get::<JsFunction, _, _>(&mut cx, "parse")?;

    #[cfg(not(windows))]
    let guard = options
        .report_file
        .is_some()
        .then(|| pprof::ProfilerGuardBuilder::default().build())
        .transpose()
        .or_else(|err| cx.throw_error(err.to_string()))?;

    if options.neon {
        group.bench_function("neon", |b| {
            b.iter(|| {
                cx.execute_scoped(|mut cx| {
                    criterion::black_box(neon::serialize::<JsValue, _, _>(&mut cx, pokedex))
                })
            })
        });
    }

    if options.json {
        group.bench_function("serde_json", |b| {
            b.iter(|| {
                cx.execute_scoped(|mut cx| {
                    let s = serde_json::to_string(pokedex).or_throw(&mut cx)?;
                    let s = cx.string(s).upcast();

                    criterion::black_box(parse.call(&mut cx, parse, [s]))
                })
            })
        });
    }

    group.finish();

    #[cfg(not(windows))]
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

fn deserialize(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let options = cx
        .argument_opt(0)
        .unwrap_or_else(|| cx.empty_object().upcast());

    let options = neon::deserialize::<BenchOptions, _, _>(&mut cx, options)?;
    let mut c = Criterion::default();
    let mut group = c.benchmark_group("Deserialize");
    let pokedex = neon::serialize::<JsValue, _, _>(&mut cx, pokemon::pokedex())?;

    group.measurement_time(Duration::from_secs(10));

    let stringify = cx
        .global()
        .get::<JsObject, _, _>(&mut cx, "JSON")?
        .get::<JsFunction, _, _>(&mut cx, "stringify")?;

    #[cfg(not(windows))]
    let guard = options
        .report_file
        .is_some()
        .then(|| pprof::ProfilerGuardBuilder::default().build())
        .transpose()
        .or_else(|err| cx.throw_error(err.to_string()))?;

    if options.neon {
        group.bench_function("neon", |b| {
            b.iter(|| {
                cx.execute_scoped(|mut cx| {
                    criterion::black_box(neon::deserialize::<pokemon::Pokedex, _, _>(
                        &mut cx, pokedex,
                    ))
                })
            })
        });
    }

    if options.json {
        group.bench_function("serde_json", |b| {
            b.iter(|| {
                cx.execute_scoped(|mut cx| {
                    let pokedex = stringify
                        .call(&mut cx, stringify, [pokedex])?
                        .downcast_or_throw::<JsString, _>(&mut cx)?
                        .value(&mut cx);

                    criterion::black_box(
                        serde_json::from_str::<pokemon::Pokedex>(&pokedex).or_throw(&mut cx),
                    )
                })
            })
        });
    }

    group.finish();

    #[cfg(not(windows))]
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

fn arguments(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let mut c = Criterion::default();
    let mut group = c.benchmark_group("Arguments");

    group.measurement_time(Duration::from_secs(10));

    let a = cx.number(1).upcast();
    let b = cx.number(2).upcast();

    let manual = JsFunction::new(&mut cx, |mut cx| {
        let a = cx.argument::<JsNumber>(0)?.value(&mut cx);
        let b = cx.argument::<JsNumber>(0)?.value(&mut cx);

        Ok(cx.number(a + b))
    })?;

    let partial = JsFunction::new(&mut cx, |mut cx| {
        let (a, b): (f64, f64) = cx.deserialize_args()?;

        Ok(cx.number(a + b))
    })?;

    let full = JsFunction::new(&mut cx, |mut cx| {
        let (a, b): (f64, f64) = cx.deserialize_args()?;

        neon::serialize::<JsValue, _, _>(&mut cx, &(a + b))
    })?;

    group.bench_function("manual", |bencher| {
        bencher.iter(|| manual.call(&mut cx, manual, [a, b]))
    });

    group.bench_function("partial", |bencher| {
        bencher.iter(|| partial.call(&mut cx, partial, [a, b]))
    });

    group.bench_function("full", |bencher| {
        bencher.iter(|| full.call(&mut cx, full, [a, b]))
    });

    group.finish();

    Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("serialize", serialize)?;
    cx.export_function("deserialize", deserialize)?;
    cx.export_function("arguments", arguments)?;

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
