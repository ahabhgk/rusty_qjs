use std::ptr::NonNull;

use crate::{
  runtime::{JSRuntime, JsRuntime},
  support::Opaque,
};

extern "C" {
  fn JS_NewContext(rt: *mut JSRuntime) -> *mut JSContext;
  fn JS_FreeContext(s: *mut JSContext);
  fn JS_DupContext(ctx: *mut JSContext) -> *mut JSContext;
}

#[repr(C)]
#[derive(Debug)]
pub struct JSContext(Opaque);

impl JSContext {
  pub unsafe fn new(rt: *mut JSRuntime) -> *mut Self {
    JS_NewContext(rt)
  }

  pub unsafe fn free(&mut self) {
    JS_FreeContext(self);
  }

  pub unsafe fn dup(&mut self) -> *mut Self {
    JS_DupContext(self)
  }
}

#[derive(Debug)]
pub struct JsContext(pub NonNull<JSContext>);

impl JsContext {
  pub fn new(rt: &mut JsRuntime) -> Self {
    let ctx = unsafe { JSContext::new(rt.0.as_mut()) };
    Self::from_raw(ctx)
  }

  pub fn from_raw(ctx: *mut JSContext) -> Self {
    let ctx = NonNull::new(ctx).unwrap();
    Self(ctx)
  }

  pub fn dup(&mut self) -> Self {
    let ctx = unsafe { self.0.as_mut().dup() };
    Self::from_raw(ctx)
  }
}

impl Drop for JsContext {
  fn drop(&mut self) {
    unsafe { self.0.as_mut().free() };
  }
}
