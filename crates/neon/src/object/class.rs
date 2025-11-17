use crate::{
    context::Cx,
    handle::{Handle, Root},
    object::Object,
    result::{JsResult, NeonResult},
    types::JsFunction,
};

#[doc(hidden)]
pub trait ClassInternal {
    fn local<'cx>(cx: &mut Cx<'cx>) -> NeonResult<ClassMetadata<'cx>>;
    fn create<'cx>(cx: &mut Cx<'cx>) -> NeonResult<ClassMetadata<'cx>>;
}

/// A trait defining a Neon class.
///
/// **This should not be implemented directly.** Instead, use the [`class`](crate::macros::class)
/// attribute macro to define a class, which will automatically implement this trait.
pub trait Class: ClassInternal {
    /// The class name.
    fn name() -> String;

    /// The constructor function for the class.
    fn constructor<'cx>(cx: &mut Cx<'cx>) -> JsResult<'cx, JsFunction>;
}

#[doc(hidden)]
pub struct ClassMetadata<'cx> {
    external_constructor: Handle<'cx, JsFunction>,
    internal_constructor: Handle<'cx, JsFunction>,
}

pub fn new_class_metadata<'cx>(
    external: Handle<'cx, JsFunction>,
    internal: Handle<'cx, JsFunction>,
) -> ClassMetadata<'cx> {
    ClassMetadata {
        external_constructor: external,
        internal_constructor: internal,
    }
}

impl<'cx> ClassMetadata<'cx> {
    pub fn constructor(&self) -> Handle<'cx, JsFunction> {
        self.external_constructor
    }

    pub(crate) fn internal_constructor(&self) -> Handle<'cx, JsFunction> {
        self.internal_constructor
    }

    #[doc(hidden)]
    pub fn root<'cx2>(&self, cx: &mut Cx<'cx2>) -> RootClassMetadata {
        RootClassMetadata {
            external_constructor: self.external_constructor.root(cx),
            internal_constructor: self.internal_constructor.root(cx),
        }
    }
}

#[doc(hidden)]
pub struct RootClassMetadata {
    pub external_constructor: Root<JsFunction>,
    pub internal_constructor: Root<JsFunction>,
}

// Since it's just a pair of Root which are both Send, we can mark it as such.
unsafe impl Send for RootClassMetadata {}

impl RootClassMetadata {
    pub fn to_inner<'a, 'cx: 'a>(&'a self, cx: &'a mut Cx<'cx>) -> ClassMetadata<'cx> {
        ClassMetadata {
            external_constructor: self.external_constructor.to_inner(cx),
            internal_constructor: self.internal_constructor.to_inner(cx),
        }
    }
}
