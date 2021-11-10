use crate::{context::JsContext, value::error::JsError};

use std::{
  ffi::c_void,
  ptr::{self, NonNull},
};

#[derive(Debug)]
pub struct JsRuntime(pub NonNull<libquickjs_sys::JSRuntime>);

impl Default for JsRuntime {
  fn default() -> Self {
    let rt = unsafe { libquickjs_sys::JS_NewRuntime() };
    let runtime = NonNull::new(rt).unwrap();
    Self(runtime)
  }
}

impl JsRuntime {
  // TODO: see rusty_v8, and write the bindings manually
  pub unsafe fn set_host_promise_rejection_tracker(
    &mut self,
    tracker: libquickjs_sys::JSHostPromiseRejectionTracker,
    opaque: *mut c_void,
  ) {
    libquickjs_sys::JS_SetHostPromiseRejectionTracker(
      self.0.as_mut(),
      tracker,
      opaque,
    )
  }

  pub fn execute_pending_job(&mut self) -> Result<bool, JsError> {
    let pctx = &mut ptr::null_mut();
    let res =
      unsafe { libquickjs_sys::JS_ExecutePendingJob(self.0.as_mut(), pctx) };
    match res {
      0 => Ok(false),
      1 => Ok(true),
      2.. => panic!("JS_ExecutePendingJob never return >1"),
      _ => {
        let mut pctx = JsContext::from_raw(*pctx);
        Err(pctx.get_exception().into())
      }
    }
  }

  pub fn free(&mut self) {
    unsafe { libquickjs_sys::JS_FreeRuntime(self.0.as_mut()) };
  }
}
