use neon::prelude::*;

register_module!(mut cx, {
    let s = cx.string("Hello, World!");

    println!("{}", s.value(&mut cx));
    cx.export_value("greeting", s)?;

    Ok(())
});
