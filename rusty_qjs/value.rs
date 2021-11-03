use std::{
  ffi::{CStr, CString},
  ptr::{self, NonNull},
};

use crate::{context::JsContext, error::JsError};

pub struct JsValue {
  pub val: libquickjs_sys::JSValue,
  pub ctx: NonNull<libquickjs_sys::JSContext>,
}

// #[repr(C)]
// #[derive(Copy, Clone)]
// pub union JSValueUnion {
//     pub int32: i32,
//     pub float64: f64,
//     pub ptr: *mut ::std::os::raw::c_void,
//     _bindgen_union_align: u64,
// }
// #[repr(C)]
// #[derive(Copy, Clone)]
// pub struct JSValue {
//     pub u: JSValueUnion,
//     pub tag: i64,
// }
// impl Debug for JsValue {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     write!(
//       f,
//       r#"JsValue {{
//         context: {:p},
//         inner: {{
//           u: {{
//             ptr: {:p}
//           }},
//           tag: {:?},
//         }},
//       }}"#,
//       self.context,
//       unsafe { self.inner.u.ptr },
//       self.inner.tag,
//     )
//   }
// }

impl From<JsValue> for String {
  fn from(mut value: JsValue) -> Self {
    let ptr = unsafe {
      libquickjs_sys::JS_ToCStringLen2(
        value.ctx.as_mut(),
        ptr::null_mut(),
        value.val,
        0,
      ) as *mut _
    };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    unsafe { libquickjs_sys::JS_FreeCString(value.ctx.as_mut(), ptr) };
    let s = cstr.to_str().unwrap();
    s.to_owned()
  }
}

impl JsValue {
  pub fn free(&mut self) {
    unsafe { libquickjs_sys::JS_FreeValue(self.ctx.as_mut(), self.val) };
  }

  pub fn new(ctx: &mut JsContext, val: libquickjs_sys::JSValue) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let ctx = NonNull::new(ctx).unwrap();
    Self { ctx, val }
  }

  pub fn new_object(ctx: &mut JsContext) -> Self {
    let obj = unsafe { libquickjs_sys::JS_NewObject(ctx.0.as_mut()) };
    Self::new(ctx, obj)
  }

  pub fn new_c_function(
    ctx: &mut JsContext,
    func: *mut libquickjs_sys::JSCFunction,
    name: &str,
    len: i32,
  ) -> Self {
    let name_cstring = CString::new(name).unwrap();
    let val = unsafe {
      libquickjs_sys::JS_NewCFunction(
        ctx.0.as_mut(),
        func,
        name_cstring.as_ptr(),
        len,
      )
    };
    Self::new(ctx, val)
  }

  pub fn new_undefined(ctx: &mut JsContext) -> Self {
    let val = libquickjs_sys::JSValue {
      u: libquickjs_sys::JSValueUnion { int32: 0 },
      tag: libquickjs_sys::JS_TAG_UNDEFINED.into(),
    };
    Self::new(ctx, val)
  }

  pub fn get_property_str(&mut self, prop: &str) -> Self {
    let ctx = unsafe { self.ctx.as_mut() };
    let prop_cstring = CString::new(prop).unwrap();
    let val = unsafe {
      libquickjs_sys::JS_GetPropertyStr(ctx, self.val, prop_cstring.as_ptr())
    };
    Self { ctx: self.ctx, val }
  }

  pub fn set_property_str(
    &mut self,
    prop: &str,
    value: JsValue,
  ) -> Result<bool, JsError> {
    let ctx = unsafe { self.ctx.as_mut() };
    let prop_cstring = CString::new(prop).unwrap();
    let result = unsafe {
      libquickjs_sys::JS_SetPropertyStr(
        ctx,
        self.val,
        prop_cstring.as_ptr(),
        value.val,
      )
    };
    match result {
      -1 => {
        let mut ctx = JsContext(NonNull::new(ctx).unwrap());
        Err(ctx.get_exception().into())
      }
      0 => Ok(false),
      1 => Ok(true),
      _ => panic!("JS_SetPropertyStr return unexpected"),
    }
  }

  pub fn is_error(&mut self) -> bool {
    unsafe { libquickjs_sys::JS_IsError(self.ctx.as_mut(), self.val) == 1 }
  }

  pub fn is_exception(&self) -> bool {
    unsafe { libquickjs_sys::JS_IsException(self.val) }
  }

  pub fn is_undefined(&self) -> bool {
    unsafe { libquickjs_sys::JS_IsUndefined(self.val) }
  }
}
