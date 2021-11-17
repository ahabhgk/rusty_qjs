use std::{
  ops::{Deref, DerefMut},
  ptr::{self, NonNull},
};

use crate::{support::Opaque, Error, JSContext, JSValue};

extern "C" {
  fn JS_NewRuntime() -> *mut JSRuntime;
  fn JS_FreeRuntime(rt: *mut JSRuntime);
  fn JS_SetHostPromiseRejectionTracker(
    rt: *mut JSRuntime,
    cb: JSHostPromiseRejectionTracker,
    opaque: *mut libc::c_void,
  );
  fn JS_ExecutePendingJob(
    rt: *mut JSRuntime,
    pctx: *mut *mut JSContext,
  ) -> libc::c_int;
}

pub type JSHostPromiseRejectionTracker = ::std::option::Option<
  unsafe extern "C" fn(
    ctx: *mut JSContext,
    promise: JSValue,
    reason: JSValue,
    is_handled: libc::c_int,
    opaque: *mut libc::c_void,
  ),
>;

#[repr(C)]
#[derive(Debug, Copy, Clone)] // Clone?
pub struct JSRuntime(Opaque);

impl JSRuntime {
  pub fn new() -> OwnedJSRuntime {
    let rt = unsafe { JS_NewRuntime() };
    let rt = NonNull::new(rt).unwrap();
    OwnedJSRuntime(rt)
  }

  /// Set callback to handle host promise rejection
  ///
  /// # Safety
  ///
  /// TODO: will use fn mapping or proc macro to abstract the unsafe
  pub unsafe fn set_host_promise_rejection_tracker(
    &mut self,
    tracker: JSHostPromiseRejectionTracker,
    opaque: *mut libc::c_void,
  ) {
    JS_SetHostPromiseRejectionTracker(self, tracker, opaque);
  }

  pub fn execute_pending_job(&mut self) -> Result<bool, Error> {
    let pctx = &mut ptr::null_mut();
    let res = unsafe { JS_ExecutePendingJob(self, pctx) };
    match res {
      0 => Ok(false),
      1 => Ok(true),
      2.. => panic!("JS_ExecutePendingJob never return >1"),
      _ => {
        let pctx = *pctx;
        let pctx = unsafe { pctx.as_mut() }.unwrap();
        // Err(JSContext::get_exception(pctx).into())
        Err(JSContext::get_exception(pctx).to_error(pctx))
      }
    }
  }

  pub fn free(&mut self) {
    unsafe { JS_FreeRuntime(self) };
  }
}

pub struct OwnedJSRuntime(NonNull<JSRuntime>);

impl Drop for OwnedJSRuntime {
  fn drop(&mut self) {
    self.free();
  }
}

impl Deref for OwnedJSRuntime {
  type Target = JSRuntime;

  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref() }
  }
}

impl DerefMut for OwnedJSRuntime {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0.as_mut() }
  }
}
