use super::context::JsContext;
use std::{
  ffi::{CStr, CString},
  ptr::{self, NonNull},
};

// #[derive(Debug)]
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

impl Drop for JsValue {
  fn drop(&mut self) {
    // never use qjs::JS_FreeValue to free qjs::JS_TAG_MODULE.
    if self.val.tag == libquickjs_sys::JS_TAG_MODULE.into() {
      return;
    }
    unsafe { libquickjs_sys::JS_FreeValue(self.ctx.as_mut(), self.val) };
  }
}

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
  pub fn new(ctx: &mut JsContext, val: libquickjs_sys::JSValue) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let ctx = NonNull::new(ctx).unwrap();
    Self { ctx, val }
  }

  pub fn get_property(&mut self, prop: &str) -> Option<Self> {
    let ctx = unsafe { self.ctx.as_mut() };
    let prop_cstring = CString::new(prop).unwrap();
    let value = unsafe {
      libquickjs_sys::JS_GetPropertyStr(ctx, self.val, prop_cstring.as_ptr())
    };
    let is_undefined = unsafe { libquickjs_sys::JS_IsUndefined(value) };
    if is_undefined {
      return None;
    }
    Some(Self {
      ctx: self.ctx,
      val: value,
    })
  }

  pub fn is_error(&mut self) -> bool {
    unsafe { libquickjs_sys::JS_IsError(self.ctx.as_mut(), self.val) == 1 }
  }

  pub fn is_exception(&self) -> bool {
    unsafe { libquickjs_sys::JS_IsException(self.val) }
  }
}
