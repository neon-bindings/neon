//! Types and traits for working with JavaScript functions.

use crate::handle::Handle;
use crate::types::Value;

use smallvec::smallvec;

pub(crate) mod private;

/// The trait for specifying arguments for a function call. This trait is sealed and cannot
/// be implemented by types outside of the Neon crate.
///
/// **Note:** This trait is implemented for tuples of up to 32 JavaScript values,
/// but for the sake of brevity, only tuples up to size 8 are shown in this documentation.
pub trait Arguments<'a>: private::ArgumentsInternal<'a> {}

impl<'a, T: Value> private::ArgumentsInternal<'a> for Vec<Handle<'a, T>> {
    fn into_args_vec(self) -> private::ArgsVec<'a> {
        let mut args = smallvec![];
        for arg in self {
            args.push(arg.upcast());
        }
        args
    }
}

impl<'a, T: Value> Arguments<'a> for Vec<Handle<'a, T>> {}

impl<'a, T: Value, const N: usize> private::ArgumentsInternal<'a> for [Handle<'a, T>; N] {
    fn into_args_vec(self) -> private::ArgsVec<'a> {
        let mut args = smallvec![];
        for arg in self {
            args.push(arg.upcast());
        }
        args
    }
}

impl<'a, T: Value, const N: usize> Arguments<'a> for [Handle<'a, T>; N] {}

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
                let mut args = smallvec![];
                let ($($vprefix, )* $vname1, ) = self;
                $(args.push($vprefix.upcast());)*
                args.push($vname1.upcast());
                args
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
