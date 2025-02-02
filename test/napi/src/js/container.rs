use std::cell::RefCell;

#[neon::export]
fn create_string_ref_cell(s: String) -> RefCell<String> {
    RefCell::new(s)
}

#[neon::export]
fn read_string_ref_cell(s: &RefCell<String>) -> String {
    s.borrow().clone()
}

#[neon::export]
fn string_ref_cell_concat(lhs: &RefCell<String>, rhs: String) -> String {
    lhs.borrow().clone() + &rhs
}
