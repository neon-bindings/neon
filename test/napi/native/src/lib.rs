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
    cx.export_value("undefined", undefined)?;
    cx.export_value("null", null)?;

    Ok(())
});
