use std::{collections::HashMap, future::Future};

use neon::{event::Channel, prelude::*, types::extract::Json};

#[neon::export]
fn wrap_string(cx: &mut Cx, o: Handle<JsObject>, s: String) -> NeonResult<()> {
    neon::macro_internal::object::wrap(cx, o, s)?.or_throw(cx)
}

#[neon::export]
fn unwrap_string(cx: &mut Cx, o: Handle<JsObject>) -> NeonResult<String> {
    neon::macro_internal::object::unwrap(cx, o)?
        .cloned()
        .or_throw(cx)
}

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

    pub fn concat(&self, other: &Self) -> Self {
        Self {
            value: format!("{}{}", self.value, other.value),
        }
    }

    pub fn append(&mut self, suffix: String) {
        self.value.push_str(&suffix);
    }

    pub fn finalize<'a, C: Context<'a>>(self, _cx: &mut C) {
        println!("Finalizing Message with value: {}", self.value);
    }
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

    // Edge case: boolean const
    const IS_2D: bool = true;

    // Edge case: const expression with conditionals
    const MAX_DIMENSION: u32 = if std::mem::size_of::<u32>() == 4 {
        2147483647
    } else {
        65535
    };

    // Edge case: const expression with match
    const COORDINATE_BYTES: u32 = match std::mem::size_of::<u32>() {
        4 => 4,
        8 => 8,
        _ => 0,
    };

    // Edge case: const expression with arithmetic (can't use sqrt in const)
    const DOUBLE_100_SQUARED: u32 = 100_u32.pow(2) * 2;

    // Edge case: string with special characters
    #[neon(name = "specialString")]
    const SPECIAL_CHARS: &'static str = "Hello\nWorld\t\"quoted\"\r\n";

    // Edge case: negative number
    const NEGATIVE_OFFSET: i32 = -42;

    // Edge case: const with underscores (use u32 instead of u64 for now)
    const MAX_SAFE_INTEGER_APPROX: u32 = 2147483647;

    // Edge case: const starting with underscore (valid in JS)
    const _PRIVATE_CONST: u32 = 999;

    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> u32 {
        self.x
    }

    pub fn y(&self) -> u32 {
        self.y
    }

    pub fn distance(&self, other: &Self) -> f64 {
        let dx = (self.x as i32 - other.x as i32).pow(2);
        let dy = (self.y as i32 - other.y as i32).pow(2);
        ((dx + dy) as f64).sqrt()
    }

    pub fn midpoint(&self, other: &Self) -> Self {
        Self {
            x: (self.x + other.x) / 2,
            y: (self.y + other.y) / 2,
        }
    }

    pub fn swap_coords(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.x, &mut other.x);
        std::mem::swap(&mut self.y, &mut other.y);
    }

    pub fn move_by(&mut self, dx: u32, dy: u32) {
        self.x += dx;
        self.y += dy;
    }

    pub fn set_x(&mut self, x: u32) {
        self.x = x;
    }

    pub fn set_y(&mut self, y: u32) {
        self.y = y;
    }
}

#[derive(Debug, Default)]
pub struct StringBuffer {
    buffer: String,
}

#[neon::class]
impl StringBuffer {
    pub fn push(&mut self, s: String) {
        self.buffer.push_str(&s);
    }

    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        self.buffer.clone()
    }

    #[neon(name = "includes")]
    pub fn contains(&self, s: String) -> bool {
        self.buffer.contains(&s)
    }

    #[neon(name = "trimStart")]
    pub fn trim_start(&self) -> String {
        self.buffer.trim_start().to_string()
    }

    pub fn trim_end(&self) -> String {
        self.buffer.trim_end().to_string()
    }
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
    pub fn heavy_computation(self) -> u32 {
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
    pub fn task_with_channel(self, _ch: Channel, multiplier: i32) -> String {
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

    // Test explicit async + JSON combination
    #[neon(async, json)]
    pub fn explicit_async_json_method(
        &self,
        data: Vec<i32>,
    ) -> impl Future<Output = Vec<i32>> + 'static {
        let data_clone = data;
        async move {
            // Simulate async work with JSON serialization
            data_clone.into_iter().map(|x| x * 2).collect()
        }
    }

    #[neon(json)]
    pub async fn async_json_method(self, data: Vec<i32>) -> Vec<i32> {
        data.into_iter().map(|x| x * 2).collect()
    }
}

// Test Rust → JS path: Create class instance in Rust and return to JS
#[neon::export]
pub fn create_point_from_rust(x: u32, y: u32) -> Point {
    Point::new(x, y)
}

// Test Rust → JS with transformation
#[neon::export]
pub fn create_point_origin() -> Point {
    Point::new(Point::ORIGIN_X, Point::ORIGIN_Y)
}

// Test Rust → JS: Accept a point, transform it, and return a new point
#[neon::export]
pub fn double_point_coords(point: Point) -> Point {
    Point::new(point.x() * 2, point.y() * 2)
}

// Test class with Result return type in constructor
#[derive(Debug, Clone)]
pub struct FallibleCounter {
    value: u32,
}

#[neon::class]
impl FallibleCounter {
    pub fn new(value: u32) -> Result<Self, String> {
        if value > 100 {
            Err("Value must be <= 100".to_string())
        } else {
            Ok(Self { value })
        }
    }

    pub fn get(&self) -> u32 {
        self.value
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }
}

// Test class with context parameter in constructor (auto-inferred, no attribute needed)
#[derive(Debug, Clone)]
pub struct ContextCounter {
    value: u32,
}

type AlsoCx<'cx> = Cx<'cx>;

#[neon::class]
impl ContextCounter {
    #[neon(context)]
    pub fn new(_cx: &mut AlsoCx, value: u32) -> Self {
        // Could use context to access JavaScript values, call functions, etc.
        // Context is auto-detected because first param is &mut Cx
        Self { value }
    }

    pub fn get(&self) -> u32 {
        self.value
    }
}

// Test class with JSON in constructor
#[derive(Debug, Clone, serde::Deserialize)]
pub struct JsonConfig {
    name: String,
    count: u32,
    enabled: bool,
}

#[neon::class]
impl JsonConfig {
    #[neon(json)]
    pub fn new(config: JsonConfig) -> Self {
        config
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

// Test class combining all features: context (auto-inferred), JSON, and Result
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ValidatedConfig {
    name: String,
    count: u32,
}

#[neon::class]
impl ValidatedConfig {
    #[neon(json)]
    pub fn new(_cx: &mut Cx, config: ValidatedConfig) -> Result<Self, String> {
        // Validate the configuration
        if config.name.is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if config.count > 1000 {
            return Err("Count must be <= 1000".to_string());
        }
        Ok(config)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn count(&self) -> u32 {
        self.count
    }
}

pub struct Secret {
    pub value: String,
}

#[neon::class]
impl Secret {
    pub fn new(cx: &mut Cx, init: Handle<JsValue>) -> NeonResult<Self> {
        if let Ok(js_str) = init.downcast::<JsString, _>(cx) {
            let secret_str: String = js_str.value(cx);
            return Ok(Self { value: secret_str });
        }
        if let Ok(js_thunk) = init.downcast::<JsFunction, _>(cx) {
            let this = cx.undefined();
            let js_result: Handle<JsValue> = js_thunk.call(cx, this, vec![])?;
            let secret_str: String = js_result.to_string(cx)?.value(cx);
            return Ok(Self { value: secret_str });
        }
        Ok(Self {
            value: "default_secret".to_string(),
        })
    }

    pub fn reveal(&self) -> String {
        self.value.clone()
    }
}

pub struct Argv {
    pub args: Vec<String>,
}

#[neon::class]
impl Argv {
    #[neon(json)]
    pub fn new(cx: &mut Cx, args: Option<Vec<String>>) -> NeonResult<Self> {
        let args = if let Some(args) = args {
            args
        } else {
            // Use global_object() instead of global() to avoid swallowing exceptions
            // from property getters. The global() method internally calls get() which
            // catches PendingException and converts it to a generic error.
            let global = cx.global_object();
            let process: Handle<JsObject> = global.prop(cx, "process").get()?;
            let Json(args): Json<Vec<String>> = process.prop(cx, "argv").get()?;
            args
        };
        Ok(Self { args })
    }

    pub fn len(&self) -> u32 {
        self.args.len() as u32
    }

    pub fn get(&self, index: u32) -> Option<String> {
        self.args.get(index as usize).cloned()
    }
}

const CAROUSEL_MESSAGES: [&str; 5] = [
    "Welcome to the Neon Carousel!",
    "Enjoy seamless Rust and JavaScript integration.",
    "Experience high performance with native modules.",
    "Build robust applications with ease.",
    "Thank you for using Neon!",
];

pub struct Carousel {
    state: u32,
}

#[neon::class]
impl Carousel {
    pub fn new() -> Self {
        Self { state: 0 }
    }

    pub fn next(&mut self) -> String {
        let message = CAROUSEL_MESSAGES[self.state as usize];
        self.state = (self.state + 1) % CAROUSEL_MESSAGES.len() as u32;
        message.to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Expando;

#[neon::class]
impl Expando {
    pub fn new(cx: &mut FunctionContext) -> NeonResult<Self> {
        let this: Handle<JsObject> = cx.this()?;
        this.prop(cx, "__weirdNeonExpandoKey__").set(42)?;
        Ok(Self)
    }

    pub fn expando(self, cx: &mut FunctionContext) -> NeonResult<i32> {
        let this: Handle<JsObject> = cx.this()?;
        let value: Handle<JsNumber> = this.prop(cx, "__weirdNeonExpandoKey__").get()?;
        Ok(value.value(cx) as i32)
    }
}
