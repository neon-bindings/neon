use smallvec::SmallVec;

use crate::{context::Context, handle::Handle, result::NeonResult, types::JsValue};

pub type ArgsVec<'a> = SmallVec<[Handle<'a, JsValue>; 8]>;

/// This type marks the `TryIntoArguments` trait as sealed.
pub trait TryIntoArgumentsInternal<'cx> {
    fn try_into_args_vec<C: Context<'cx>>(self, cx: &mut C) -> NeonResult<ArgsVec<'cx>>;
}

/// This type marks the `Arguments` trait as sealed.
pub trait ArgumentsInternal<'a> {
    fn into_args_vec(self) -> ArgsVec<'a>;
}
