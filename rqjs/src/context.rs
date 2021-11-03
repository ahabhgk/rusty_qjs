use std::ptr::NonNull;

use crate::runtime::JsRuntime;

#[derive(Debug)]
pub struct JsContext(pub NonNull<libquickjs_sys::JSContext>);

impl JsContext {
  pub fn new(rt: &mut JsRuntime) -> Self {
    let ctx = unsafe { libquickjs_sys::JS_NewContext(rt.0.as_mut()) };
    Self::from_raw(ctx)
  }

  pub fn from_raw(ctx: *mut libquickjs_sys::JSContext) -> Self {
    let ctx = NonNull::new(ctx).unwrap();
    Self(ctx)
  }

  pub fn dup(&mut self) -> Self {
    let ctx = unsafe { libquickjs_sys::JS_DupContext(self.0.as_mut()) };
    Self::from_raw(ctx)
  }
}

impl Drop for JsContext {
  fn drop(&mut self) {
    unsafe { libquickjs_sys::JS_FreeContext(self.0.as_mut()) };
  }
}
