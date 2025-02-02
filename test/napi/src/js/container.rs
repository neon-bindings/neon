use neon::prelude::*;

use std::cell::{Ref, RefCell, RefMut};

#[neon::export]
fn create_string_ref_cell(s: String) -> RefCell<String> {
    RefCell::new(s)
}

#[neon::export]
fn read_string_ref_cell(s: &RefCell<String>) -> String {
    s.borrow().clone()
}

#[neon::export]
fn write_string_ref_cell(s: &RefCell<String>, value: String) {
    *s.borrow_mut() = value;
}

#[neon::export]
fn string_ref_cell_concat(lhs: &RefCell<String>, rhs: String) -> String {
    lhs.borrow().clone() + &rhs
}

#[neon::export]
fn string_ref_concat(lhs: Ref<String>, rhs: String) -> String {
    lhs.clone() + &rhs
}

#[neon::export]
fn write_string_ref(mut s: RefMut<String>, value: String) {
    *s = value;
}

#[neon::export]
fn borrow_and_then<'cx>(
    cx: &mut Cx<'cx>,
    cell: &RefCell<String>,
    f: Handle<JsFunction>,
) -> JsResult<'cx, JsString> {
    let s = cell.borrow();
    f.bind(cx).exec()?;
    Ok(cx.string(s.clone()))
}

#[neon::export]
fn borrow_mut_and_then<'cx>(
    cx: &mut Cx<'cx>,
    cell: &RefCell<String>,
    f: Handle<JsFunction>,
) -> JsResult<'cx, JsString> {
    let mut s = cell.borrow_mut();
    f.bind(cx).exec()?;
    *s = "overwritten".to_string();
    Ok(cx.string(s.clone()))
}
