use std::{
  mem,
  ops::{Deref, DerefMut},
  ptr::{self, NonNull},
};

use crate::{
  error::JSContextException,
  support::{MapFnFrom, MapFnTo, Opaque, ToCFn, UnitType},
  JSContext, JSValue,
};

extern "C" {
  fn JS_NewRuntime() -> *mut JSRuntime;
  fn JS_FreeRuntime(rt: *mut JSRuntime);
  fn JS_SetHostPromiseRejectionTracker(
    rt: *mut JSRuntime,
    cb: GenJSHostPromiseRejectionTracker,
    opaque: *mut libc::c_void,
  );
  fn JS_ExecutePendingJob(
    rt: *mut JSRuntime,
    pctx: *mut *mut JSContext,
  ) -> libc::c_int;
}

pub type GenJSHostPromiseRejectionTracker = ::std::option::Option<
  unsafe extern "C" fn(
    ctx: *mut JSContext,
    promise: JSValue,
    reason: JSValue,
    is_handled: libc::c_int,
    opaque: *mut libc::c_void,
  ),
>;

pub type JSHostPromiseRejectionTracker = extern "C" fn(
  ctx: *mut JSContext,
  promise: JSValue,
  reason: JSValue,
  is_handled: libc::c_int,
  opaque: *mut libc::c_void,
);

impl<F> MapFnFrom<F> for JSHostPromiseRejectionTracker
where
  F: UnitType + Fn(&mut JSContext, JSValue, JSValue, bool, *mut libc::c_void),
{
  fn mapping() -> Self {
    let f = |ctx: *mut JSContext,
             promise: JSValue,
             reason: JSValue,
             is_handled: libc::c_int,
             opaque: *mut libc::c_void| {
      let ctx = unsafe { ctx.as_mut() }.unwrap();
      (F::get())(ctx, promise, reason, is_handled != 0, opaque)
    };
    f.to_c_fn()
  }
}

/// JSRuntime represents a Javascript runtime corresponding to an
/// object heap. Several runtimes can exist at the same time but they
/// cannot exchange objects. Inside a given runtime, no multi-threading
/// is supported.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JSRuntime(Opaque);

impl JSRuntime {
  /// Create a new JSRuntime.
  /// use JS_NewJSRuntime internally.
  #[allow(clippy::new_ret_no_self)]
  pub fn new() -> OwnedJSRuntime {
    let rt = unsafe { JS_NewRuntime() };
    let rt = NonNull::new(rt).unwrap();
    OwnedJSRuntime(rt)
  }

  /// Set callback to handle host promise rejection.
  /// use JS_SetHostPromiseRejectionTracker internally.
  #[allow(clippy::not_unsafe_ptr_arg_deref)]
  pub fn set_host_promise_rejection_tracker(
    &mut self,
    tracker: impl MapFnTo<JSHostPromiseRejectionTracker>,
    opaque: *mut libc::c_void,
  ) {
    let tracker = tracker.map_fn_to();
    unsafe {
      JS_SetHostPromiseRejectionTracker(
        self,
        mem::transmute(tracker as *mut ()),
        opaque,
      )
    };
  }

  /// Executes next pending job, returns JSContextException if exception,
  /// false if no job pending, true if a job was executed successfully.
  /// use JS_ExecutePendingJob internally.
  pub fn execute_pending_job(&mut self) -> Result<bool, JSContextException> {
    let pctx = &mut ptr::null_mut();
    let res = unsafe { JS_ExecutePendingJob(self, pctx) };
    match res {
      0 => Ok(false),
      1 => Ok(true),
      2.. => panic!("JS_ExecutePendingJob never return >1"),
      _ => {
        let pctx = *pctx;
        let pctx = unsafe { pctx.as_mut() }.unwrap();
        let e = JSContext::get_exception(pctx);
        Err(JSContextException::from_jsvalue(pctx, e))
      }
    }
  }

  /// Free the JSRuntime. use JS_FreeRuntime internally.
  pub fn free(&mut self) {
    unsafe { JS_FreeRuntime(self) };
  }
}

/// Same as JSRuntime but gets freed when it drops.
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
