use std::ptr::NonNull;

use crate::support::Opaque;

extern "C" {
  fn JS_NewRuntime() -> *mut JSRuntime;
  fn JS_FreeRuntime(rt: *mut JSRuntime);
  // fn JS_SetHostPromiseRejectionTracker(
  //   rt: *mut JSRuntime,
  //   cb: JSHostPromiseRejectionTracker,
  //   opaque: *mut ::std::os::raw::c_void,
  // );
}

#[repr(C)]
#[derive(Debug)]
pub struct JSRuntime(Opaque);

impl JSRuntime {
  pub unsafe fn new() -> *mut Self {
    JS_NewRuntime()
  }

  pub unsafe fn free(&mut self) {
    JS_FreeRuntime(self)
  }
}

#[derive(Debug)]
pub struct JsRuntime(pub NonNull<JSRuntime>);

impl JsRuntime {
  pub fn new() -> Self {
    let rt = unsafe { JSRuntime::new() };
    let rt = NonNull::new(rt).unwrap();
    Self(rt)
  }
}

impl Drop for JsRuntime {
  fn drop(&mut self) {
    unsafe { self.0.as_mut().free() };
  }
}
