use super::{context::JsContext, error::JsError};
use libquickjs_sys as qjs;
use std::{marker::PhantomData, rc::Rc};

pub struct JsRuntime {
  inner: *mut qjs::JSRuntime,
  _marker: PhantomData<*mut qjs::JSRuntime>,
}

impl Drop for JsRuntime {
  fn drop(&mut self) {
    unsafe { qjs::JS_FreeRuntime(self.inner) };
  }
}

impl Default for JsRuntime {
  fn default() -> Self {
    let runtime = unsafe { qjs::JS_NewRuntime() };
    Self {
      inner: runtime,
      _marker: PhantomData,
    }
  }
}

impl JsRuntime {
  pub(crate) fn inner(&self) -> *mut qjs::JSRuntime {
    self.inner
  }

  pub fn execute_pending_job(&self) -> Result<bool, JsError> {
    let ctx = Rc::new(JsContext::default());
    let pctx = &mut ctx.inner();
    let res = unsafe { qjs::JS_ExecutePendingJob(self.inner, pctx) };
    match res {
      0 => Ok(false),
      1 => Ok(true),
      1.. => panic!(),
      _ => Err(ctx.into()),
    }
  }
}
