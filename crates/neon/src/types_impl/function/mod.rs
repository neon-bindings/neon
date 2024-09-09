//! Types and traits for working with JavaScript functions.

use smallvec::smallvec;

use crate::{
    context::{Context, Cx},
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult},
    types::{
        call_local,
        extract::{TryFromJs, TryIntoJs},
        private::ValueInternal,
        JsFunction, JsObject, JsValue, Value,
    },
};

pub(crate) mod private;

/// A builder for making a JavaScript function call like `parseInt("42")`.
///
/// The builder methods make it convenient to assemble the call from parts:
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsNumber> {
/// # let parse_int: Handle<JsFunction> = cx.global("parseInt")?;
/// let x: f64 = parse_int
///     .bind(&mut cx)
///     .arg("42")?
///     .apply()?;
/// # Ok(cx.number(x))
/// # }
/// ```
pub struct BindOptions<'a, 'cx: 'a> {
    pub(crate) cx: &'a mut Cx<'cx>,
    pub(crate) callee: Handle<'cx, JsValue>,
    pub(crate) this: Option<Handle<'cx, JsValue>>,
    pub(crate) args: private::ArgsVec<'cx>,
}

impl<'a, 'cx: 'a> BindOptions<'a, 'cx> {
    /// Set the value of `this` for the function call.
    pub fn this<V: Value>(&mut self, this: Handle<'cx, V>) -> &mut Self {
        self.this = Some(this.upcast());
        self
    }

    /// Replaces the arguments list with the given arguments.
    pub fn args<A: TryIntoArguments<'cx>>(&mut self, a: A) -> NeonResult<&mut Self> {
        self.args = a.try_into_args_vec(self.cx)?;
        Ok(self)
    }

    /// Add an argument to the arguments list.
    pub fn arg<A: TryIntoJs<'cx>>(&mut self, a: A) -> NeonResult<&mut Self> {
        let v = a.try_into_js(self.cx)?;
        self.args.push(v.upcast());
        Ok(self)
    }

    /// Add an argument to the arguments list, computed from a closure.
    pub fn arg_with<R, F>(&mut self, f: F) -> NeonResult<&mut Self>
    where
        R: TryIntoJs<'cx>,
        F: FnOnce(&mut Cx<'cx>) -> R,
    {
        let v = f(self.cx).try_into_js(self.cx)?;
        self.args.push(v.upcast());
        Ok(self)
    }

    /// Make the function call. If the function returns without throwing, the result value
    /// is converted to a Rust value with `TryFromJs::from_js`.
    pub fn apply<R: TryFromJs<'cx>>(&mut self) -> NeonResult<R> {
        let this = self.this.unwrap_or_else(|| self.cx.undefined().upcast());
        let v: Handle<JsValue> =
            unsafe { call_local(self.cx, self.callee.to_local(), this, &self.args)? };
        R::from_js(self.cx, v)
    }
}

/// A builder for making a JavaScript function call like `parseInt("42")`.
///
/// The builder methods make it convenient to assemble the call from parts:
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsNumber> {
/// # let parse_int: Handle<JsFunction> = cx.global("parseInt")?;
/// let x: Handle<JsNumber> = parse_int
///     .call_with(&cx)
///     .arg(cx.string("42"))
///     .apply(&mut cx)?;
/// # Ok(x)
/// # }
/// ```
#[derive(Clone)]
pub struct CallOptions<'a> {
    pub(crate) callee: Handle<'a, JsFunction>,
    pub(crate) this: Option<Handle<'a, JsValue>>,
    pub(crate) args: private::ArgsVec<'a>,
}

impl<'a> CallOptions<'a> {
    /// Set the value of `this` for the function call.
    pub fn this<V: Value>(&mut self, this: Handle<'a, V>) -> &mut Self {
        self.this = Some(this.upcast());
        self
    }

    /// Add an argument to the arguments list.
    pub fn arg<V: Value>(&mut self, arg: Handle<'a, V>) -> &mut Self {
        self.args.push(arg.upcast());
        self
    }

    /// Replaces the arguments list with the given arguments.
    pub fn args<A: Arguments<'a>>(&mut self, args: A) -> &mut Self {
        self.args = args.into_args_vec();
        self
    }

    /// Make the function call. If the function returns without throwing, the result value
    /// is downcast to the type `V`, throwing a `TypeError` if the downcast fails.
    pub fn apply<'b: 'a, V: Value, C: Context<'b>>(&self, cx: &mut C) -> JsResult<'b, V> {
        let this = self.this.unwrap_or_else(|| cx.undefined().upcast());
        let v: Handle<JsValue> = self.callee.call(cx, this, &self.args)?;
        v.downcast_or_throw(cx)
    }

    /// Make the function call for side effect, discarding the result value. This method is
    /// preferable to [`apply()`](CallOptions::apply) when the result value isn't needed,
    /// since it doesn't require specifying a result type.
    pub fn exec<'b: 'a, C: Context<'b>>(&self, cx: &mut C) -> NeonResult<()> {
        let this = self.this.unwrap_or_else(|| cx.undefined().upcast());
        self.callee.call(cx, this, &self.args)?;
        Ok(())
    }
}

/// A builder for making a JavaScript constructor call like `new Array(16)`.
///
/// The builder methods make it convenient to assemble the call from parts:
/// ```
/// # use neon::prelude::*;
/// # fn foo(mut cx: FunctionContext) -> JsResult<JsObject> {
/// # let url: Handle<JsFunction> = cx.global("URL")?;
/// let obj = url
///     .construct_with(&cx)
///     .arg(cx.string("https://neon-bindings.com"))
///     .apply(&mut cx)?;
/// # Ok(obj)
/// # }
/// ```
#[derive(Clone)]
pub struct ConstructOptions<'a> {
    pub(crate) callee: Handle<'a, JsFunction>,
    pub(crate) args: private::ArgsVec<'a>,
}

impl<'a> ConstructOptions<'a> {
    /// Add an argument to the arguments list.
    pub fn arg<V: Value>(&mut self, arg: Handle<'a, V>) -> &mut Self {
        self.args.push(arg.upcast());
        self
    }

    /// Replaces the arguments list with the given arguments.
    pub fn args<A: Arguments<'a>>(&mut self, args: A) -> &mut Self {
        self.args = args.into_args_vec();
        self
    }

    /// Make the constructor call. If the function returns without throwing, returns
    /// the resulting object.
    pub fn apply<'b: 'a, O: Object, C: Context<'b>>(&self, cx: &mut C) -> JsResult<'b, O> {
        let v: Handle<JsObject> = self.callee.construct(cx, &self.args)?;
        v.downcast_or_throw(cx)
    }
}

/// The trait for specifying values to be converted into arguments for a function call.
/// This trait is sealed and cannot be implemented by types outside of the Neon crate.
///
/// **Note:** This trait is implemented for tuples of up to 32 JavaScript values,
/// but for the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait TryIntoArguments<'cx>: private::TryIntoArgumentsInternal<'cx> {}

impl<'cx> private::TryIntoArgumentsInternal<'cx> for () {
    fn try_into_args_vec(self, _cx: &mut Cx<'cx>) -> NeonResult<private::ArgsVec<'cx>> {
        Ok(smallvec![])
    }
}

macro_rules! impl_into_arguments {
    {
        [ $(($tprefix:ident, $vprefix:ident), )* ];
        [];
    } => {};

    {
        [ $(($tprefix:ident, $vprefix:ident), )* ];
        [ $(#[$attr1:meta])? ($tname1:ident, $vname1:ident), $($(#[$attrs:meta])? ($tnames:ident, $vnames:ident), )* ];
    } => {
        $(#[$attr1])?
        impl<'cx, $($tprefix: TryIntoJs<'cx> + 'cx, )* $tname1: TryIntoJs<'cx> + 'cx> private::TryIntoArgumentsInternal<'cx> for ($($tprefix, )* $tname1, ) {
            fn try_into_args_vec(self, cx: &mut Cx<'cx>) -> NeonResult<private::ArgsVec<'cx>> {
                let ($($vprefix, )* $vname1, ) = self;
                Ok(smallvec![ $($vprefix.try_into_js(cx)?.upcast(),)* $vname1.try_into_js(cx)?.upcast() ])
             }
         }

         $(#[$attr1])?
         impl<'cx, $($tprefix: TryIntoJs<'cx> + 'cx, )* $tname1: TryIntoJs<'cx> + 'cx> TryIntoArguments<'cx> for ($($tprefix, )* $tname1, ) {}

         impl_into_arguments! {
             [ $(($tprefix, $vprefix), )* ($tname1, $vname1), ];
             [ $($(#[$attrs])? ($tnames, $vnames), )* ];
         }
     };
}

impl_into_arguments! {
    [];
    [
        (V1, v1),
        (V2, v2),
        (V3, v3),
        (V4, v4),
        (V5, v5),
        (V6, v6),
        (V7, v7),
        (V8, v8),
        #[doc(hidden)]
        (V9, v9),
        #[doc(hidden)]
        (V10, v10),
        #[doc(hidden)]
        (V11, v11),
        #[doc(hidden)]
        (V12, v12),
        #[doc(hidden)]
        (V13, v13),
        #[doc(hidden)]
        (V14, v14),
        #[doc(hidden)]
        (V15, v15),
        #[doc(hidden)]
        (V16, v16),
        #[doc(hidden)]
        (V17, v17),
        #[doc(hidden)]
        (V18, v18),
        #[doc(hidden)]
        (V19, v19),
        #[doc(hidden)]
        (V20, v20),
        #[doc(hidden)]
        (V21, v21),
        #[doc(hidden)]
        (V22, v22),
        #[doc(hidden)]
        (V23, v23),
        #[doc(hidden)]
        (V24, v24),
        #[doc(hidden)]
        (V25, v25),
        #[doc(hidden)]
        (V26, v26),
        #[doc(hidden)]
        (V27, v27),
        #[doc(hidden)]
        (V28, v28),
        #[doc(hidden)]
        (V29, v29),
        #[doc(hidden)]
        (V30, v30),
        #[doc(hidden)]
        (V31, v31),
        #[doc(hidden)]
        (V32, v32),
    ];
}

/// The trait for specifying arguments for a function call. This trait is sealed and cannot
/// be implemented by types outside of the Neon crate.
///
/// **Note:** This trait is implemented for tuples of up to 32 JavaScript values,
/// but for the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait Arguments<'a>: private::ArgumentsInternal<'a> {}

impl<'a> private::ArgumentsInternal<'a> for () {
    fn into_args_vec(self) -> private::ArgsVec<'a> {
        smallvec![]
    }
}

impl<'a> Arguments<'a> for () {}

macro_rules! impl_arguments {
    {
        [ $(($tprefix:ident, $vprefix:ident), )* ];
        [];
    } => {};

    {
        [ $(($tprefix:ident, $vprefix:ident), )* ];
        [ $(#[$attr1:meta])? ($tname1:ident, $vname1:ident), $($(#[$attrs:meta])? ($tnames:ident, $vnames:ident), )* ];
    } => {
        $(#[$attr1])?
        impl<'a, $($tprefix: Value, )* $tname1: Value> private::ArgumentsInternal<'a> for ($(Handle<'a, $tprefix>, )* Handle<'a, $tname1>, ) {
            fn into_args_vec(self) -> private::ArgsVec<'a> {
                let ($($vprefix, )* $vname1, ) = self;
                smallvec![$($vprefix.upcast(),)* $vname1.upcast()]
             }
         }

         $(#[$attr1])?
         impl<'a, $($tprefix: Value, )* $tname1: Value> Arguments<'a> for ($(Handle<'a, $tprefix>, )* Handle<'a, $tname1>, ) {}

         impl_arguments! {
             [ $(($tprefix, $vprefix), )* ($tname1, $vname1), ];
             [ $($(#[$attrs])? ($tnames, $vnames), )* ];
         }
     };
 }

impl_arguments! {
    [];
    [
        (V1, v1),
        (V2, v2),
        (V3, v3),
        (V4, v4),
        (V5, v5),
        (V6, v6),
        (V7, v7),
        (V8, v8),
        #[doc(hidden)]
        (V9, v9),
        #[doc(hidden)]
        (V10, v10),
        #[doc(hidden)]
        (V11, v11),
        #[doc(hidden)]
        (V12, v12),
        #[doc(hidden)]
        (V13, v13),
        #[doc(hidden)]
        (V14, v14),
        #[doc(hidden)]
        (V15, v15),
        #[doc(hidden)]
        (V16, v16),
        #[doc(hidden)]
        (V17, v17),
        #[doc(hidden)]
        (V18, v18),
        #[doc(hidden)]
        (V19, v19),
        #[doc(hidden)]
        (V20, v20),
        #[doc(hidden)]
        (V21, v21),
        #[doc(hidden)]
        (V22, v22),
        #[doc(hidden)]
        (V23, v23),
        #[doc(hidden)]
        (V24, v24),
        #[doc(hidden)]
        (V25, v25),
        #[doc(hidden)]
        (V26, v26),
        #[doc(hidden)]
        (V27, v27),
        #[doc(hidden)]
        (V28, v28),
        #[doc(hidden)]
        (V29, v29),
        #[doc(hidden)]
        (V30, v30),
        #[doc(hidden)]
        (V31, v31),
        #[doc(hidden)]
        (V32, v32),
    ];
}
