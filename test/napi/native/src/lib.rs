use neon::prelude::*;

register_module!(mut cx, {
    let s = cx.string("Hello, World!");
    let s = s.value(&mut cx);

    println!("{}", s);

    Ok(())
});
