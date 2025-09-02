use neon::{prelude::*, types::extract::Instance};

#[derive(Debug, Clone)]
pub struct Message {
    value: String,
}

#[neon::class]
impl Message {
    pub fn new(value: String) -> Self {
        Self { value }
    }

    pub fn read(&self) -> &str {
        &self.value
    }

    pub fn concat(&self, other: Instance<Self>) -> Instance<Self> {
        Instance(Self { value: format!("{}{}", self.value, other.0.value) })
    }
}

impl Finalize for Message {
    fn finalize<'cx, C: Context<'cx>>(self, _cx: &mut C) { }
}

#[derive(Debug, Clone)]
pub struct Point {
    x: u32,
    y: u32,
}

#[neon::class]
impl Point {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn distance(&self, other: Instance<Self>) -> f64 {
        let dx = (self.x as i32 - other.x() as i32).pow(2);
        let dy = (self.y as i32 - other.y() as i32).pow(2);
        ((dx + dy) as f64).sqrt()
    }
}

impl Finalize for Point {
    fn finalize<'cx, C: Context<'cx>>(self, _cx: &mut C) { }
}

/*
impl Class for Message {
    fn create_class<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction> {
        let wrap = JsFunction::new(cx, |mut cx| {
            let this: Handle<JsObject> = cx.argument(0)?;
            let value: String = cx.argument::<JsString>(1)?.value(&mut cx);
            wrap(&mut cx, this, Message::constructor(value))?.or_throw(&mut cx)?;
            Ok(cx.undefined())
        });

        let read = JsFunction::new(cx, |mut cx| {
            let this: Handle<JsObject> = cx.this()?;
            let message: &Message = unwrap(&mut cx, this)?.or_throw(&mut cx)?;
            let result = cx.string(&message.value);
            Ok(result)
        });

        const CLASS_MAKER_SCRIPT: &str = r#"
(function makeClass(wrap, read) {
  class Message {
    constructor(s) {
      wrap(this, s);
    }
  }
  const prototype = Message.prototype;
  prototype.read = read;
  return Message;
})
"#;

        let src = cx.string(CLASS_MAKER_SCRIPT);
        let constructor: Handle<JsFunction> = neon::reflect::eval(cx, src)?
            .downcast(cx)
            .or_throw(cx)?;
        constructor
            .bind(cx)
            .arg(wrap)?
            .arg(read)?
            .call()
    }
}
*/
