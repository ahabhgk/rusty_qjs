use std::{
  fmt, mem,
  ops::{Deref, DerefMut},
  ptr::NonNull,
};

use crate::context::JSContext;

pub trait QuickjsRc {
  fn free(&mut self, ctx: &mut JSContext);

  fn dup(&self, ctx: &mut JSContext) -> Self;
}

pub struct Local<'ctx, T: QuickjsRc> {
  value: NonNull<T>,
  context: &'ctx mut JSContext,
}

impl<T: QuickjsRc> Drop for Local<'_, T> {
  fn drop(&mut self) {
    unsafe { self.value.as_mut() }.free(self.context);
  }
}

impl<T: QuickjsRc> Deref for Local<'_, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { self.value.as_ref() }
  }
}

impl<T: QuickjsRc> DerefMut for Local<'_, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.value.as_mut() }
  }
}

impl<T: QuickjsRc + fmt::Debug> fmt::Debug for Local<'_, T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&**self, f)
  }
}

impl<'ctx, T: QuickjsRc> Local<'ctx, T> {
  pub fn new(ctx: &'ctx mut JSContext, rc: T) -> Self {
    // this way to get *mut T, 'as' has potential unsafety
    let ptr = Box::into_raw(Box::new(rc));
    let value = NonNull::new(ptr).unwrap();
    Self {
      value,
      context: ctx,
    }
  }

  pub fn to_qjsrc(self) -> T {
    let nn = self.value;
    mem::forget(self);
    // Safety: the NonNull pointer is created by `Box::into_raw` in `Local::from`
    let b = unsafe { Box::from_raw(nn.as_ptr()) };
    *b
  }

  pub fn dup(&mut self, ctx: &'ctx mut JSContext) -> Self {
    let rc = unsafe { self.value.as_ref() }.dup(self.context);
    Self::new(ctx, rc)
  }
}

#[cfg(test)]
mod tests {
  // use crate::{context::JsContext, runtime::JsRuntime, value::JsValue};

  // #[test]
  // fn new_with_same_context() {
  //   let rt = JsRuntime::default();
  //   let ctx = JsContext::new(&rt);

  //   let o1 = JsValue::new_object(&ctx);
  //   let o2 = JsValue::new_object(&ctx);

  //   assert_eq!(o1.raw_context, o2.raw_context);
  // }
}
