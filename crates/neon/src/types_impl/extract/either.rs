use std::{any, error, fmt};

use either::Either;

use crate::{
    context::Cx,
    handle::Handle,
    object::Object,
    result::{JsResult, NeonResult},
    types::{
        JsError, JsValue,
        extract::{TryFromJs, TryIntoJs, private},
    },
};

impl<'cx, L, R> TryFromJs<'cx> for Either<L, R>
where
    L: TryFromJs<'cx>,
    R: TryFromJs<'cx>,
{
    type Error = Error<L::Error, R::Error>;

    fn try_from_js(
        cx: &mut Cx<'cx>,
        v: Handle<'cx, JsValue>,
    ) -> NeonResult<Result<Self, Self::Error>> {
        let left = match L::try_from_js(cx, v)? {
            Ok(l) => return Ok(Ok(Either::Left(l))),
            Err(l) => l,
        };

        let right = match R::try_from_js(cx, v)? {
            Ok(r) => return Ok(Ok(Either::Right(r))),
            Err(r) => r,
        };

        Ok(Err(Error::new::<L, R>(left, right)))
    }
}

impl<'cx, L, R> TryIntoJs<'cx> for Either<L, R>
where
    L: TryIntoJs<'cx>,
    R: TryIntoJs<'cx>,
{
    type Value = JsValue;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        match self {
            Either::Left(v) => v.try_into_js(cx).map(|v| v.upcast()),
            Either::Right(v) => v.try_into_js(cx).map(|v| v.upcast()),
        }
    }
}

impl<L, R> private::Sealed for Either<L, R> {}

#[derive(Debug)]
pub struct Error<L, R> {
    left: (&'static str, L),
    right: (&'static str, R),
}

impl<'cx, L, R> Error<L, R> {
    fn new<LT, RT>(left: L, right: R) -> Self
    where
        LT: TryFromJs<'cx, Error = L>,
        RT: TryFromJs<'cx, Error = R>,
    {
        Self {
            left: (any::type_name::<LT>(), left),
            right: (any::type_name::<RT>(), right),
        }
    }
}

impl<L, R> fmt::Display for Error<L, R>
where
    L: fmt::Display,
    R: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Either::Left: {}", self.left.1)?;
        write!(f, "Either::Right: {}", self.right.1)
    }
}

impl<L, R> error::Error for Error<L, R>
where
    L: error::Error,
    R: error::Error,
{
}

impl<'cx, L, R> TryIntoJs<'cx> for Error<L, R>
where
    L: TryIntoJs<'cx>,
    R: TryIntoJs<'cx>,
{
    type Value = JsError;

    fn try_into_js(self, cx: &mut Cx<'cx>) -> JsResult<'cx, Self::Value> {
        let err = JsError::type_error(
            cx,
            format!("expected either {} or {}", self.left.0, self.right.0,),
        )?;

        err.prop(cx, "left").set(self.left.1)?;
        err.prop(cx, "right").set(self.right.1)?;

        Ok(err)
    }
}

impl<L, R> private::Sealed for Error<L, R> {}
