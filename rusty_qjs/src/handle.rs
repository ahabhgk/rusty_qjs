use std::{
  mem,
  ops::{Deref, DerefMut},
  ptr::NonNull,
};

pub trait QuickjsRc {
  fn free(&mut self);

  fn dup(&self) -> Self;
}

pub struct Local<T: QuickjsRc>(pub NonNull<T>);

impl<T: QuickjsRc> Drop for Local<T> {
  fn drop(&mut self) {
    unsafe { self.0.as_mut() }.free()
  }
}

impl<T: QuickjsRc> Deref for Local<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref() }
  }
}

impl<T: QuickjsRc> DerefMut for Local<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0.as_mut() }
  }
}

impl<T: QuickjsRc> From<Reference<T>> for Local<T> {
  fn from(rc: Reference<T>) -> Self {
    Self(rc.0)
  }
}

impl<T: QuickjsRc> Local<T> {
  // FIXME!!!
  pub fn new(v: T) -> Self {
    Self::from_raw(&v as *const T)
  }

  // FIXME!!!
  pub fn from_raw(v: *const T) -> Self {
    let v = NonNull::new(v as *mut _).unwrap();
    Self(v)
  }

  pub fn to_reference(self) -> Reference<T> {
    Reference::from(self)
  }
}

pub struct Reference<T: QuickjsRc>(NonNull<T>);

impl<T: QuickjsRc> QuickjsRc for Reference<T> {
  fn free(&mut self) {
    unsafe { self.0.as_mut() }.free()
  }

  fn dup(&self) -> Self {
    let v = unsafe { self.0.as_ref() }.dup();
    Self::new(v)
  }
}

impl<T: QuickjsRc> Deref for Reference<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref() }
  }
}

impl<T: QuickjsRc> DerefMut for Reference<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0.as_mut() }
  }
}

impl<T: QuickjsRc> From<Local<T>> for Reference<T> {
  fn from(lc: Local<T>) -> Self {
    let v = lc.0;
    mem::forget(lc);
    Self(v)
  }
}

impl<T: QuickjsRc> Reference<T> {
  pub fn new(v: T) -> Self {
    Self::from_raw(&v)
  }

  pub fn from_raw(v: *const T) -> Self {
    let v = NonNull::new(v as *mut _).unwrap();
    Self(v)
  }

  pub fn to_local(self) -> Local<T> {
    Local::from(self)
  }
}

#[cfg(test)]
mod tests {
  use crate::{
    context::JsContext, handle::Local, runtime::JsRuntime, value::JsValue,
  };

  #[test]
  fn wtf_new_object() {
    let rt = JsRuntime::default();
    let ctx = JsContext::new(&rt);

    fn new_non_local_object(ctx: &JsContext) -> JsValue {
      let raw_context = ctx.raw_context;
      let obj = unsafe { libquickjs_sys::JS_NewObject(raw_context) };
      JsValue::from_raw(raw_context, obj)
    }
    let value1 = new_non_local_object(&ctx);
    let value1_raw = &value1 as *const JsValue;
    let value2 = Local::from_raw(value1_raw);

    let value3 = new_non_local_object(&ctx);
    let value3 = Local::new(value3);

    fn new_local_object(ctx: &JsContext) -> Local<JsValue> {
      let raw_context = ctx.raw_context;
      let obj = unsafe { libquickjs_sys::JS_NewObject(raw_context) };
      let v = JsValue::from_raw(raw_context, obj);
      Local::from_raw(&v as *const JsValue)
    }

    let value4 = new_local_object(&ctx);

    unsafe {
      libquickjs_sys::DEBUG_log(value1.raw_context);
      libquickjs_sys::DEBUG_log(value2.raw_context);

      libquickjs_sys::DEBUG_log(value3.raw_context);

      libquickjs_sys::DEBUG_log(value4.raw_context);
    };
  }

  // see cli/core#L127
  fn wtf_dump_error() {}
}
