use std::{ffi::CString, ptr::NonNull};

use crate::context::JsContext;

pub struct JsValue {
  val: libquickjs_sys::JSValue,
  ctx: NonNull<libquickjs_sys::JSContext>,
}

impl JsValue {
  pub fn new_big_int64(ctx: &mut JsContext, v: i64) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let val = unsafe { libquickjs_sys::JS_NewBigInt64(ctx, v) };
    let ctx = NonNull::new(ctx).unwrap();
    Self { val, ctx }
  }

  pub fn new_big_uint64(ctx: &mut JsContext, v: u64) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let val = unsafe { libquickjs_sys::JS_NewBigUint64(ctx, v) };
    let ctx = NonNull::new(ctx).unwrap();
    Self { val, ctx }
  }

  pub fn free(&mut self) {
    unsafe { libquickjs_sys::JS_FreeValue(self.ctx.as_mut(), self.val) };
  }

  pub fn get_property_str(&mut self, prop: &str) -> Self {
    let ctx = unsafe { self.ctx.as_mut() };
    let prop = CString::new(prop).unwrap();
    let val = unsafe {
      libquickjs_sys::JS_GetPropertyStr(ctx, self.val, prop.as_ptr())
    };
    Self { val, ctx: self.ctx }
  }

  // pub fn set_property_str(
  //   &mut self,
  //   prop: &str,
  //   value: libquickjs_sys::JSValue,
  // ) -> Result<bool, JsError> {
  //   let ctx = unsafe { self.ctx.as_mut() };
  //   let prop_cstring = CString::new(prop).unwrap();
  //   let result = unsafe {
  //     libquickjs_sys::JS_SetPropertyStr(
  //       ctx,
  //       self.val,
  //       prop_cstring.as_ptr(),
  //       value,
  //     )
  //   };
  //   match result {
  //     -1 => {
  //       let mut ctx = JsContext(NonNull::new(ctx).unwrap());
  //       Err(ctx.get_exception().into())
  //     }
  //     0 => Ok(false),
  //     1 => Ok(true),
  //     _ => panic!("JS_SetPropertyStr return unexpected"),
  //   }
  // }
}
