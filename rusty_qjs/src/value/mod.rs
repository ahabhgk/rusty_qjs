use std::{
  ffi::{CStr, CString},
  ptr,
};

use crate::{
  context::JsContext,
  error::Error,
  handle::{Local, QuickjsRc},
};

type JsFunction = extern "C" fn(
  *mut libquickjs_sys::JSContext,
  libquickjs_sys::JSValue,
  i32,
  *mut libquickjs_sys::JSValue,
) -> libquickjs_sys::JSValue;

pub struct JsValue {
  pub raw_value: libquickjs_sys::JSValue,
  pub raw_context: *mut libquickjs_sys::JSContext,
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

impl QuickjsRc for JsValue {
  fn free(&mut self) {
    unsafe { libquickjs_sys::JS_FreeValue(self.raw_context, self.raw_value) };
  }

  fn dup(&self) -> Self {
    let raw_value =
      unsafe { libquickjs_sys::JS_DupValue(self.raw_context, self.raw_value) };
    Self::from_raw(self.raw_context, raw_value)
  }
}

impl From<Local<JsValue>> for String {
  fn from(value: Local<JsValue>) -> Self {
    let value = value.to_reference();
    let ptr = unsafe {
      libquickjs_sys::JS_ToCStringLen2(
        value.raw_context,
        ptr::null_mut(),
        value.raw_value,
        0,
      ) as *mut _
    };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    unsafe { libquickjs_sys::JS_FreeCString(value.raw_context, ptr) };
    let s = cstr.to_str().unwrap();
    s.to_owned()
  }
}

impl From<Local<JsValue>> for Error {
  fn from(value: Local<JsValue>) -> Self {
    let (name, message, stack) = if value.is_error() {
      let name = value.get_property_str("name").into();
      let message = value.get_property_str("message").into();
      let stack = value.get_property_str("stack").into();

      (name, message, stack)
    } else {
      let message = String::from(value);
      ("Exception".to_owned(), message, "".to_owned())
    };

    Self::JSContextError {
      name,
      stack,
      message,
    }
  }
}

impl JsValue {
  pub fn from_raw(
    raw_context: *mut libquickjs_sys::JSContext,
    raw_value: libquickjs_sys::JSValue,
  ) -> Self {
    Self {
      raw_context,
      raw_value,
    }
  }

  pub fn new_object(ctx: &JsContext) -> Self {
    let raw_context = ctx.raw_context;
    let obj = unsafe { libquickjs_sys::JS_NewObject(raw_context) };
    Self::from_raw(raw_context, obj)
  }

  pub fn new_function(
    ctx: &JsContext,
    func: JsFunction,
    name: &str,
    len: i32,
  ) -> Local<Self> {
    let raw_context = ctx.raw_context;
    let name_cstring = CString::new(name).unwrap();
    let val = unsafe {
      libquickjs_sys::JS_NewCFunction(
        raw_context,
        std::mem::transmute(func as *mut ()),
        name_cstring.as_ptr(),
        len,
      )
    };
    Local::new(Self::from_raw(raw_context, val))
  }

  pub fn new_undefined(ctx: &JsContext) -> Local<Self> {
    let raw_context = ctx.raw_context;
    let val = libquickjs_sys::JSValue {
      u: libquickjs_sys::JSValueUnion { int32: 0 },
      tag: libquickjs_sys::JS_TAG_UNDEFINED.into(),
    };
    Local::new(Self::from_raw(raw_context, val))
  }

  pub fn get_property_str(&self, prop: &str) -> Local<Self> {
    let prop_cstring = CString::new(prop).unwrap();
    let raw_value = unsafe {
      libquickjs_sys::JS_GetPropertyStr(
        self.raw_context,
        self.raw_value,
        prop_cstring.as_ptr(),
      )
    };
    Local::new(Self::from_raw(self.raw_context, raw_value))
  }

  pub fn set_property_str(
    &self,
    prop: &str,
    value: Local<JsValue>,
  ) -> Result<bool, Error> {
    let value = value.to_reference();
    let prop_cstring = CString::new(prop).unwrap();
    let result = unsafe {
      libquickjs_sys::JS_SetPropertyStr(
        self.raw_context,
        self.raw_value,
        prop_cstring.as_ptr(),
        value.raw_value,
      )
    };
    match result {
      -1 => Err(Local::new(JsContext::from_raw(self.raw_context).get_exception()).into()),
      0 => Ok(false),
      1 => Ok(true),
      _ => panic!("JS_SetPropertyStr return unexpected"),
    }
  }

  pub fn is_error(&self) -> bool {
    unsafe { libquickjs_sys::JS_IsError(self.raw_context, self.raw_value) == 1 }
  }

  pub fn is_exception(&self) -> bool {
    unsafe { libquickjs_sys::JS_IsException(self.raw_value) }
  }

  pub fn is_undefined(&self) -> bool {
    unsafe { libquickjs_sys::JS_IsUndefined(self.raw_value) }
  }
}
