use crate::handle::Handle;
use crate::types::JsValue;

use smallvec::SmallVec;

pub type ArgsVec<'a> = SmallVec<[Handle<'a, JsValue>; 8]>;

/// This type marks the `Arguments` trait as sealed.
pub trait ArgumentsInternal<'a> {
    fn append(self, args: &mut ArgsVec<'a>);
}
