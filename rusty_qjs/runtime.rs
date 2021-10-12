use super::{context::JsContext, error::JsError};
use libquickjs_sys as qjs;
use std::{ffi::c_void, marker::PhantomData};

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

  // TODO: see rusty_v8, and write the bindings manually
  pub unsafe fn set_host_promise_rejection_tracker(
    &self,
    tracker: qjs::JSHostPromiseRejectionTracker,
    opaque: *mut c_void,
  ) {
    qjs::JS_SetHostPromiseRejectionTracker(self.inner, tracker, opaque)
  }

  pub fn execute_pending_job(&self) -> Result<bool, JsError> {
    let runtime = JsRuntime::default();
    let ctx = JsContext::new(&runtime);
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
