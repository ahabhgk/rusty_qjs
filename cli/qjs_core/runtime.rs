use libquickjs_sys as qjs;
use std::marker::PhantomData;

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
}
