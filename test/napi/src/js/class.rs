use std::{cell::RefCell, collections::HashMap, future::Future, rc::Rc};

use neon::{event::Channel, prelude::*, types::extract::Instance};

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
        Instance(Self {
            value: format!("{}{}", self.value, other.0.value),
        })
    }
}

impl Finalize for Message {
    fn finalize<'cx, C: Context<'cx>>(self, _cx: &mut C) {}
}

#[derive(Debug, Clone)]
pub struct Point {
    x: u32,
    y: u32,
}

#[neon::class]
impl Point {
    // Basic const properties
    const ORIGIN_X: u32 = 0;
    const ORIGIN_Y: u32 = 0;

    // Const property with custom name
    #[neon(name = "maxCoordinate")]
    const MAX_COORD: u32 = 1000;

    // Const property with simple JSON (string slice)
    #[neon(json)]
    const DEFAULT_MESSAGE: &'static [&'static str] = &["hello", "point"];

    // Test complex const expressions
    const COMPUTED_VALUE: u32 = 10 + 20 + 12;
    const SIZE_OF_F64: u32 = std::mem::size_of::<f64>() as u32;
    const STRING_LENGTH: u32 = "complex".len() as u32;

    // Test const expressions that use type information
    const SELF_SIZE: u32 = std::mem::size_of::<Self>() as u32;
    const POINT_ALIGNMENT: u32 = std::mem::align_of::<Point>() as u32;

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
    fn finalize<'cx, C: Context<'cx>>(self, _cx: &mut C) {}
}

#[derive(Debug, Clone, Default)]
pub struct StringBuffer {
    buffer: Rc<RefCell<String>>,
}

#[neon::class]
impl StringBuffer {
    pub fn push(&self, s: String) {
        self.buffer.borrow_mut().push_str(&s);
    }

    pub fn to_string(&self) -> String {
        self.buffer.borrow().clone()
    }

    #[neon(name = "includes")]
    pub fn contains(&self, s: String) -> bool {
        self.buffer.borrow().contains(&s)
    }

    #[neon(name = "trimStart")]
    pub fn trim_start(&self) -> String {
        self.buffer.borrow_mut().trim_start().to_string()
    }

    pub fn trim_end(&self) -> String {
        self.buffer.borrow_mut().trim_end().to_string()
    }
}

impl Finalize for StringBuffer {
    fn finalize<'cx, C: Context<'cx>>(self, _cx: &mut C) {}
}

// Test class with async methods
#[derive(Debug, Clone)]
pub struct AsyncClass {
    value: String,
}

#[neon::class]
impl AsyncClass {
    // Simple const property
    const DEFAULT_TIMEOUT: u32 = 5000;

    // Const property with custom name and simple JSON
    #[neon(name = "version", json)]
    const VERSION_NUMBERS: &'static [u32] = &[1, 0, 0];

    pub fn new(value: String) -> Self {
        Self { value }
    }

    // This would fail to compile if we tried to use &self:
    // pub async fn async_method(&self, suffix: String) -> String {
    //     format!("{}{}", self.value, suffix)
    // }

    // Async method that takes ownership (required for 'static Future)
    pub async fn async_method(self, suffix: String) -> String {
        // Simulate async work
        format!("{}{}", self.value, suffix)
    }

    // Task method for CPU-intensive work
    #[neon(task)]
    pub fn heavy_computation(&self) -> u32 {
        // Simulate CPU-intensive work
        let mut result = 0;
        for i in 0..100 {
            result += i;
        }
        result
    }

    // Normal synchronous method for comparison
    pub fn sync_method(&self) -> String {
        self.value.clone()
    }

    // JSON method for testing serde serialization
    #[neon(json)]
    pub fn json_method(&self, data: Vec<String>) -> HashMap<String, String> {
        let mut result = HashMap::new();
        result.insert("class_value".to_string(), self.value.clone());
        result.insert("input_count".to_string(), data.len().to_string());
        result.insert(
            "first_item".to_string(),
            data.first().unwrap_or(&"none".to_string()).clone(),
        );
        result
    }

    // Explicit async method - developer controls cloning
    #[neon(async)]
    pub fn explicit_async_method(&self, multiplier: i32) -> impl Future<Output = String> + 'static {
        // Can do sync work here on main thread
        let base_value = format!("Processing: {}", self.value);

        // Must return 'static Future, so can't borrow &self
        async move {
            // Simulate async work
            format!("{} * {}", base_value, multiplier)
        }
    }

    // Explicit async method that clones by choice
    #[neon(async)]
    pub fn explicit_async_clone(&self, suffix: String) -> impl Future<Output = String> + 'static {
        // Developer explicitly chooses to clone for 'static Future
        let value_clone = self.value.clone();

        async move { format!("{}{}", value_clone, suffix) }
    }

    // Method with context parameter (sync)
    pub fn method_with_context<'a>(
        &self,
        cx: &mut FunctionContext<'a>,
        multiplier: i32,
    ) -> JsResult<'a, JsNumber> {
        let result = self.value.len() as f64 * multiplier as f64;
        Ok(cx.number(result))
    }

    // Method with explicit context attribute
    #[neon(context)]
    pub fn method_with_explicit_context(&self, _ctx: &mut Cx, suffix: String) -> String {
        format!("{}:{}", self.value, suffix)
    }

    // Task method with Channel parameter
    #[neon(task)]
    pub fn task_with_channel(&self, _ch: Channel, multiplier: i32) -> String {
        // Channel is available for background tasks
        format!("Task with channel: {} * {}", self.value, multiplier)
    }

    // AsyncFn method with Channel parameter
    pub async fn async_fn_with_channel(self, _ch: Channel, suffix: String) -> String {
        // Channel is available for async functions
        format!("AsyncFn with channel: {}{}", self.value, suffix)
    }

    #[allow(unused_variables)]
    // Method with this parameter (should auto-detect)
    pub fn method_with_this(&self, this: Handle<JsObject>, data: String) -> String {
        // Access to both Rust instance and JavaScript object
        format!(
            "Instance: {}, JS object available, data: {}",
            self.value, data
        )
    }

    // Method with explicit this attribute
    #[neon(this)]
    pub fn method_with_explicit_this(&self, _js_obj: Handle<JsObject>, suffix: String) -> String {
        format!("Explicit this: {}{}", self.value, suffix)
    }

    // Method with context and this
    #[neon(this)]
    pub fn method_with_context_and_this<'a>(
        &self,
        cx: &mut FunctionContext<'a>,
        _this: Handle<JsObject>,
        multiplier: i32,
    ) -> JsResult<'a, JsNumber> {
        let result = self.value.len() as f64 * multiplier as f64;
        Ok(cx.number(result))
    }

    // Performance test methods
    pub fn simple_method(&self, x: i32) -> i32 {
        x * 2
    }

    #[neon(json)]
    pub fn json_method_perf(&self, data: Vec<i32>) -> Vec<i32> {
        data.into_iter().map(|x| x * 2).collect()
    }

    pub fn context_method_perf(&self, _cx: &mut FunctionContext, x: i32) -> i32 {
        x * 3
    }
}

impl Finalize for AsyncClass {
    fn finalize<'cx, C: Context<'cx>>(self, _cx: &mut C) {}
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
