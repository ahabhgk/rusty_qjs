use crate::value::JsValue;

use super::{context::JsContext, error::JsError};
use libquickjs_sys as qjs;
use std::{marker::PhantomData, ptr, rc::Rc};

#[derive(Debug)]
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

  pub fn set_host_promise_rejection_tracker<F>(
    &self,
    tracker: qjs::JSHostPromiseRejectionTracker,
  ) {
    unsafe {
      qjs::JS_SetHostPromiseRejectionTracker(
        self.inner,
        tracker,
        ptr::null_mut(),
      )
    };
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
