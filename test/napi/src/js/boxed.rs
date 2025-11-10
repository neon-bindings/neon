use std::cell::RefCell;

use neon::{prelude::*, types::extract::Boxed};

pub struct Person {
    name: String,
}

impl Finalize for Person {}

impl Person {
    fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    fn greet(&self) -> String {
        format!("Hello, {}!", self.name)
    }

    fn set_name(&mut self, name: impl ToString) {
        self.name = name.to_string();
    }
}

pub fn person_new(mut cx: FunctionContext) -> JsResult<JsBox<Person>> {
    let name = cx.argument::<JsString>(0)?.value(&mut cx);
    let person = Person::new(name);

    Ok(cx.boxed(person))
}

pub fn person_greet(mut cx: FunctionContext) -> JsResult<JsString> {
    let person = cx.argument::<JsBox<Person>>(0)?;
    let greeting = cx.string(person.greet());

    Ok(greeting)
}

pub fn ref_person_new(mut cx: FunctionContext) -> JsResult<JsValue> {
    let name = cx.argument::<JsString>(0)?.value(&mut cx);
    let person = RefCell::new(Person::new(name));

    Ok(cx.boxed(person).upcast())
}

pub fn ref_person_greet(mut cx: FunctionContext) -> JsResult<JsString> {
    let person = cx.argument::<JsBox<RefCell<Person>>>(0)?;
    let greeting = cx.string(person.borrow().greet());

    Ok(greeting)
}

pub fn ref_person_set_name(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let person = cx.argument::<JsBox<RefCell<Person>>>(0)?;
    let name = cx.argument::<JsString>(1)?.value(&mut cx);

    person.borrow_mut().set_name(name);

    Ok(cx.undefined())
}

pub fn ref_person_fail(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let person = cx.argument::<JsBox<RefCell<Person>>>(0)?;
    let _borrow = person.borrow();
    let _borrow_mut = person.borrow_mut();

    Ok(cx.undefined())
}

pub fn external_unit(mut cx: FunctionContext) -> JsResult<JsBox<()>> {
    Ok(cx.boxed(()))
}

#[neon::export]
fn create_boxed_string(s: String) -> Boxed<String> {
    Boxed(s)
}

#[neon::export]
fn boxed_string_concat(Boxed(this): Boxed<String>, rhs: String) -> String {
    this + &rhs
}

#[neon::export]
// N.B.: Intentionally including unused `cx` and not using tuple struct pattern to test the macro
fn boxed_string_repeat(_cx: &mut FunctionContext, this: Boxed<String>, n: f64) -> String {
    this.0.repeat(n as usize)
}
