use std::ptr::NonNull;

#[derive(Debug)]
pub struct JsRuntime(pub NonNull<libquickjs_sys::JSRuntime>);

impl JsRuntime {
  pub fn new() -> Self {
    let rt = unsafe { libquickjs_sys::JS_NewRuntime() };
    let rt = NonNull::new(rt).unwrap();
    Self(rt)
  }
}

impl Drop for JsRuntime {
  fn drop(&mut self) {
    unsafe { libquickjs_sys::JS_FreeRuntime(self.0.as_mut()) };
  }
}
