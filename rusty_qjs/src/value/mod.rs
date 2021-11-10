pub mod error;

use std::{
  ffi::{CStr, CString},
  ptr::{self, NonNull},
};

use crate::context::JsContext;

use self::error::JsError;

type JsFunction = extern "C" fn(
  *mut libquickjs_sys::JSContext,
  libquickjs_sys::JSValue,
  i32,
  *mut libquickjs_sys::JSValue,
) -> libquickjs_sys::JSValue;

pub struct JsValue {
  pub raw_value: libquickjs_sys::JSValue,
  pub context: NonNull<libquickjs_sys::JSContext>,
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
        value.context.as_mut(),
        ptr::null_mut(),
        value.raw_value,
        0,
      ) as *mut _
    };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    unsafe { libquickjs_sys::JS_FreeCString(value.context.as_mut(), ptr) };
    let s = cstr.to_str().unwrap();
    s.to_owned()
  }
}

impl JsValue {
  pub fn free(&mut self) {
    unsafe {
      libquickjs_sys::JS_FreeValue(self.context.as_mut(), self.raw_value)
    };
  }

  pub fn from_raw(
    raw_context: *mut libquickjs_sys::JSContext,
    raw_value: libquickjs_sys::JSValue,
  ) -> Self {
    Self {
      context: NonNull::new(raw_context).unwrap(),
      raw_value,
    }
  }

  pub fn new_object(ctx: &mut JsContext) -> Self {
    let raw_context = unsafe { ctx.0.as_mut() };
    let obj = unsafe { libquickjs_sys::JS_NewObject(raw_context) };
    Self::from_raw(raw_context, obj)
  }

  pub fn new_function(
    ctx: &mut JsContext,
    func: JsFunction,
    name: &str,
    len: i32,
  ) -> Self {
    let raw_context = unsafe { ctx.0.as_mut() };
    let name_cstring = CString::new(name).unwrap();
    let val = unsafe {
      libquickjs_sys::JS_NewCFunction(
        raw_context,
        std::mem::transmute(func as *mut ()),
        name_cstring.as_ptr(),
        len,
      )
    };
    Self::from_raw(raw_context, val)
  }

  pub fn new_undefined(ctx: &JsContext) -> Self {
    let val = libquickjs_sys::JSValue {
      u: libquickjs_sys::JSValueUnion { int32: 0 },
      tag: libquickjs_sys::JS_TAG_UNDEFINED.into(),
    };
    let raw_context = ctx.0.as_ptr();
    Self::from_raw(raw_context, val)
  }

  pub fn get_property_str(&mut self, prop: &str) -> Self {
    let raw_context = unsafe { self.context.as_mut() };
    let prop_cstring = CString::new(prop).unwrap();
    let raw_value = unsafe {
      libquickjs_sys::JS_GetPropertyStr(
        raw_context,
        self.raw_value,
        prop_cstring.as_ptr(),
      )
    };
    Self::from_raw(raw_context, raw_value)
  }

  pub fn set_property_str(
    &mut self,
    prop: &str,
    value: JsValue,
  ) -> Result<bool, JsError> {
    let raw_context = unsafe { self.context.as_mut() };
    let prop_cstring = CString::new(prop).unwrap();
    let result = unsafe {
      libquickjs_sys::JS_SetPropertyStr(
        raw_context,
        self.raw_value,
        prop_cstring.as_ptr(),
        value.raw_value,
      )
    };
    match result {
      -1 => {
        let mut ctx = JsContext::from_raw(raw_context);
        Err(ctx.get_exception().into())
      }
      0 => Ok(false),
      1 => Ok(true),
      _ => panic!("JS_SetPropertyStr return unexpected"),
    }
  }

  pub fn is_error(&mut self) -> bool {
    unsafe {
      libquickjs_sys::JS_IsError(self.context.as_mut(), self.raw_value) == 1
    }
  }

  pub fn is_exception(&self) -> bool {
    unsafe { libquickjs_sys::JS_IsException(self.raw_value) }
  }

  pub fn is_undefined(&self) -> bool {
    unsafe { libquickjs_sys::JS_IsUndefined(self.raw_value) }
  }
}
