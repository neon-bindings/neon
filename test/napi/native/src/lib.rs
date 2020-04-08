use neon::prelude::*;

register_module!(|mut cx| {
    let greeting = cx.string("Hello, World!");
    let greeting_copy = greeting.value(&mut cx);
    let greeting_copy = cx.string(greeting_copy);

    cx.export_value("greeting", greeting)?;
    cx.export_value("greetingCopy", greeting_copy)?;

    // Global singletons.
    let undefined = cx.undefined();
    let null = cx.null();
    let b_true = cx.boolean(true);
    let b_false = cx.boolean(false);

    assert_eq!(b_true.value(&mut cx), true);
    assert_eq!(b_false.value(&mut cx), false);

    cx.export_value("undefined", undefined)?;
    cx.export_value("null", null)?;
    cx.export_value("true", b_true)?;
    cx.export_value("false", b_false)?;

    let one = cx.number(1);
    let two = cx.number(2.1);
    assert_eq!(one.value(&mut cx), 1.0);
    assert_eq!(two.value(&mut cx), 2.1);
    cx.export_value("one", one)?;
    cx.export_value("two", two)?;

    // Plain objects.
    let rust_created = cx.empty_object();
    {
        let a = cx.number(1);
        // set at name
        rust_created.set(&mut cx, "a", a)?;
        // set at index
        rust_created.set(&mut cx, 0, a)?;
    }
    {
        let whatever = cx.boolean(true);
        rust_created.set(&mut cx, "whatever", whatever)?;
    }

    assert_eq!({
        let v: Handle<JsNumber> = rust_created.get(&mut cx, "a")?.downcast_or_throw(&mut cx)?;
        v.value(&mut cx)
    }, 1.0f64);
    assert_eq!({
        let v: Handle<JsNumber> = rust_created.get(&mut cx, 0)?.downcast_or_throw(&mut cx)?;
        v.value(&mut cx)
    }, 1.0f64);
    assert_eq!({
        let v: Handle<JsBoolean> = rust_created.get(&mut cx, "whatever")?.downcast_or_throw(&mut cx)?;
        v.value(&mut cx)
    }, true);

    let property_names = rust_created.get_own_property_names(&mut cx)?
        .to_vec(&mut cx)?
        .into_iter()
        .map(|value| {
            let string: Handle<JsString> = value.downcast_or_throw(&mut cx)?;
            Ok(string.value(&mut cx))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(property_names, &["0", "a", "whatever"]);

    cx.export_value("rustCreated", rust_created)?;

    Ok(())
});
