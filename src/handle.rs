use std::{
  fmt, mem,
  ops::{Deref, DerefMut},
  ptr::NonNull,
};

use crate::JSContext;

pub trait QuickjsRc {
  fn free(&mut self, ctx: &mut JSContext);

  fn dup(&self, ctx: &mut JSContext) -> Self;
}

pub struct Local<T: QuickjsRc> {
  value: NonNull<T>,
  pub context: *mut JSContext,
}

impl<T: QuickjsRc> Drop for Local<T> {
  fn drop(&mut self) {
    let ctx = unsafe { self.context.as_mut() }.unwrap();
    unsafe { self.value.as_mut() }.free(ctx);
  }
}

impl<T: QuickjsRc> Deref for Local<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { self.value.as_ref() }
  }
}

impl<T: QuickjsRc> DerefMut for Local<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.value.as_mut() }
  }
}

impl<T: QuickjsRc + fmt::Debug> fmt::Debug for Local<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Debug::fmt(&**self, f)
  }
}

impl<T: QuickjsRc> Local<T> {
  pub fn from_qjsrc(ctx: &mut JSContext, rc: T) -> Self {
    // this way to get *mut T, 'as' has potential unsafety
    let ptr = Box::into_raw(Box::new(rc));
    let value = NonNull::new(ptr).unwrap();
    Self {
      value,
      context: ctx,
      // context: PhantomData,
    }
  }

  pub fn to_qjsrc(self) -> T {
    let nn = self.value;
    mem::forget(self);
    // Safety: the NonNull pointer is created by `Box::into_raw` in `Local::from`
    let b = unsafe { Box::from_raw(nn.as_ptr()) };
    *b
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
