use std::{
  fmt, mem,
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

impl<T: QuickjsRc + fmt::Debug> fmt::Debug for Local<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&**self, f)
  }
}

impl<T: QuickjsRc> Local<T> {
  pub fn new(v: T) -> Self {
    // this way to get *mut T, 'as' has potential unsafety
    let p = Box::into_raw(Box::new(v));
    let v = NonNull::new(p).unwrap();
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

impl<T: QuickjsRc + fmt::Debug> fmt::Debug for Reference<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&**self, f)
  }
}

impl<T: QuickjsRc> Reference<T> {
  pub fn new(v: T) -> Self {
    let p = Box::into_raw(Box::new(v));
    let v = NonNull::new(p).unwrap();
    Self(v)
  }

  pub fn to_local(self) -> Local<T> {
    Local::from(self)
  }
}

#[cfg(test)]
mod tests {
  use crate::{context::JsContext, runtime::JsRuntime, value::JsValue};

  #[test]
  fn new_with_same_context() {
    let rt = JsRuntime::default();
    let ctx = JsContext::new(&rt);

    let o1 = JsValue::new_object(&ctx);
    let o2 = JsValue::new_object(&ctx);

    assert_eq!(o1.raw_context, o2.raw_context);
  }
}
