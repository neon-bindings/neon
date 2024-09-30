use std::marker::PhantomData;

use crate::{
    context::Context,
    handle::{Handle, Root},
    object::Object,
    result::{JsResult, NeonResult},
    thread::LocalKey,
    types::{JsFunction, JsObject},
};

static CONSOLE: LocalKey<Root<JsObject>> = LocalKey::new();
static LOG: LocalKey<Root<JsFunction>> = LocalKey::new();
static ERROR: LocalKey<Root<JsFunction>> = LocalKey::new();

pub struct Console<'a, 'cx: 'a, C: Context<'cx>> {
    pub(crate) cx: &'a mut C,
    pub(crate) marker: PhantomData<&'cx ()>,
}

impl<'a, 'cx: 'a, C: Context<'cx>> Console<'a, 'cx, C> {
    fn memo<T, F>(
        &mut self,
        cache: &'static LocalKey<Root<T>>,
        get_container: F,
        key: &str,
    ) -> JsResult<'cx, T>
    where
        T: Object,
        F: FnOnce(&mut Self) -> JsResult<'cx, JsObject>,
    {
        let container = get_container(self)?;
        let v = cache.get_or_try_init(self.cx, |cx| {
            let v: Handle<T> = container.get(cx, key)?;
            Ok(v.root(cx))
        })?;
        Ok(v.to_inner(self.cx))
    }

    pub(crate) fn new(cx: &'a mut C) -> Self {
        Self {
            cx,
            marker: PhantomData,
        }
    }

    fn console_object(&mut self) -> JsResult<'cx, JsObject> {
        self.memo(&CONSOLE, |c| Ok(c.cx.global_object()), "console")
    }

    fn log_function(&mut self) -> JsResult<'cx, JsFunction> {
        self.memo(&LOG, |c| c.console_object(), "log")
    }

    fn error_function(&mut self) -> JsResult<'cx, JsFunction> {
        let console = self.console_object()?;
        let function = ERROR.get_or_try_init(self.cx, |cx| {
            let log: Handle<JsFunction> = console.get(cx, "error")?;
            Ok(log.root(cx))
        })?;
        Ok(function.to_inner(self.cx))
    }

    // FIXME: when we land #1056, this can get simplified
    // FIXME: msg should be a generic TryIntoJs
    pub fn log(&mut self, msg: &str) -> NeonResult<()> {
        let function = self.log_function()?;
        let console = self.console_object()?;
        let msg = self.cx.string(msg);
        let args = vec![msg.upcast()];
        function.call(self.cx, console, args)?;
        Ok(())
    }

    // FIXME: msg should be a generic TryIntoJs
    pub fn error(&mut self, msg: &str) -> NeonResult<()> {
        let function = self.error_function()?;
        let console = self.console_object()?;
        let msg = self.cx.string(msg);
        let args = vec![msg.upcast()];
        function.call(self.cx, console, args)?;
        Ok(())
    }
}
