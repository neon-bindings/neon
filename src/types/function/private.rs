use smallvec::SmallVec;

use crate::{handle::Handle, types::JsValue};

pub type ArgsVec<'a> = SmallVec<[Handle<'a, JsValue>; 8]>;

/// This type marks the `Arguments` trait as sealed.
pub trait ArgumentsInternal<'a> {
    fn into_args_vec(self) -> ArgsVec<'a>;
}
