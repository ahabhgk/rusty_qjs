use super::context::JsContext;
use libquickjs_sys as qjs;
use std::{
  ffi::{CStr, CString},
  fmt::Debug,
  ptr,
  rc::Rc,
};

#[derive(Clone)]
pub struct JsValue {
  context: Rc<JsContext>,
  inner: qjs::JSValue,
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
impl Debug for JsValue {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      r#"JsValue {{
        context: {:p},
        inner: {{
          u: {{
            ptr: {:p}
          }},
          tag: {:?},
        }},
      }}"#,
      self.context,
      unsafe { self.inner.u.ptr },
      self.inner.tag,
    )
  }
}

impl Drop for JsValue {
  fn drop(&mut self) {
    // never use qjs::JS_FreeValue to free qjs::JSValue.
    if self.inner.tag == qjs::JS_TAG_MODULE.into() {
      return;
    }
    unsafe { qjs::JS_FreeValue(self.context.inner(), self.inner) };
  }
}

impl From<JsValue> for String {
  fn from(value: JsValue) -> Self {
    let ptr = unsafe {
      qjs::JS_ToCStringLen2(
        value.context.inner(),
        ptr::null_mut(),
        value.inner,
        0,
      ) as *mut _
    };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    unsafe { qjs::JS_FreeCString(value.context.inner(), ptr) };
    let s = cstr.to_str().unwrap();
    s.to_owned()
  }
}

impl JsValue {
  pub fn from_qjs(ctx: Rc<JsContext>, value: qjs::JSValue) -> Self {
    Self {
      context: ctx,
      inner: value,
    }
  }

  pub fn get_property(&self, ctx: Rc<JsContext>, prop: &str) -> Option<Self> {
    let prop_cstring = CString::new(prop).unwrap();
    let value = unsafe {
      qjs::JS_GetPropertyStr(ctx.inner(), self.inner, prop_cstring.as_ptr())
    };
    let is_undefined = unsafe { qjs::JS_IsUndefined(value) };
    if is_undefined {
      return None;
    }
    Some(Self::from_qjs(ctx, value))
  }

  pub(crate) fn inner(&self) -> qjs::JSValue {
    self.inner
  }

  pub fn is_error(&self, ctx: Rc<JsContext>) -> bool {
    unsafe { qjs::JS_IsError(ctx.inner(), self.inner()) == 1 }
  }

  pub fn is_exception(&self) -> bool {
    unsafe { qjs::JS_IsException(self.inner()) }
  }
}
