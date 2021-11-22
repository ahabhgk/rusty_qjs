use std::{
  ffi::{CStr, CString},
  fmt,
};

use crate::{
  error::JSContextException,
  support::{cstr_to_string, jsbool_to_bool},
  JSContext, QuickjsRc,
};

extern "C" {
  fn JS_FreeValue_real(ctx: *mut JSContext, v: JSValue);
  fn JS_DupValue_real(ctx: *mut JSContext, v: JSValue) -> JSValue;
  fn JS_NewObject(ctx: *mut JSContext) -> JSValue;
  fn JS_NewCFunction_real(
    ctx: *mut JSContext,
    func: *mut JSCFunction,
    name: *const ::std::os::raw::c_char,
    length: ::std::os::raw::c_int,
  ) -> JSValue;
  fn JS_ToCStringLen_real(
    ctx: *mut JSContext,
    plen: *mut libc::size_t,
    val1: JSValue,
  ) -> *const libc::c_char;
  fn JS_ToCString_real(
    ctx: *mut JSContext,
    val1: JSValue,
  ) -> *const libc::c_char;
  fn JS_FreeCString(ctx: *mut JSContext, ptr: *const libc::c_char);
  fn JS_IsError(ctx: *mut JSContext, val: JSValue) -> libc::c_int;
  fn JS_IsException_real(v: JSValue) -> bool;
  fn JS_IsUndefined_real(v: JSValue) -> bool;
  fn JS_GetPropertyStr(
    ctx: *mut JSContext,
    this_obj: JSValue,
    prop: *const libc::c_char,
  ) -> JSValue;
  fn JS_SetPropertyStr(
    ctx: *mut JSContext,
    this_obj: JSValue,
    prop: *const libc::c_char,
    val: JSValue,
  ) -> libc::c_int;
}

type JSCFunction = ::std::option::Option<
  unsafe extern "C" fn(
    ctx: *mut JSContext,
    this_val: JSValue,
    argc: libc::c_int,
    argv: *mut JSValue,
  ) -> JSValue,
>;

pub type JSFunction =
  extern "C" fn(*mut JSContext, JSValue, i32, *mut JSValue) -> JSValue;

#[repr(C)]
#[derive(Copy, Clone)]
union JSValueUnion {
  int32: i32,
  float64: f64,
  ptr: *mut libc::c_void,
  _union_align: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct JSValue {
  u: JSValueUnion,
  tag: i64,
}

impl JSValue {
  // const JS_TAG_FIRST: i32 = -11;
  const JS_TAG_BIG_DECIMAL: i32 = -11;
  const JS_TAG_BIG_INT: i32 = -10;
  const JS_TAG_BIG_FLOAT: i32 = -9;
  const JS_TAG_SYMBOL: i32 = -8;
  const JS_TAG_STRING: i32 = -7;
  const JS_TAG_MODULE: i32 = -3;
  const JS_TAG_FUNCTION_BYTECODE: i32 = -2;
  const JS_TAG_OBJECT: i32 = -1;
  const JS_TAG_INT: i32 = 0;
  const JS_TAG_BOOL: i32 = 1;
  const JS_TAG_NULL: i32 = 2;
  const JS_TAG_UNDEFINED: i32 = 3;
  const JS_TAG_UNINITIALIZED: i32 = 4;
  const JS_TAG_CATCH_OFFSET: i32 = 5;
  const JS_TAG_EXCEPTION: i32 = 6;
  const JS_TAG_FLOAT64: i32 = 7;
}

impl fmt::Debug for JSValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let tag = match self.tag as i32 {
      Self::JS_TAG_BIG_DECIMAL => "BigDecimal",
      Self::JS_TAG_BIG_INT => "BigInt",
      Self::JS_TAG_BIG_FLOAT => "BigFloat",
      Self::JS_TAG_SYMBOL => "Symbol",
      Self::JS_TAG_STRING => "String",
      Self::JS_TAG_MODULE => "Module (internal)",
      Self::JS_TAG_FUNCTION_BYTECODE => "FunctionBytecode (internal)",
      Self::JS_TAG_OBJECT => "Object",
      Self::JS_TAG_INT => "Int",
      Self::JS_TAG_BOOL => "Bool",
      Self::JS_TAG_NULL => "Null",
      Self::JS_TAG_UNDEFINED => "Undefined",
      Self::JS_TAG_UNINITIALIZED => "Uninitialized",
      Self::JS_TAG_CATCH_OFFSET => "CatchOffset",
      Self::JS_TAG_EXCEPTION => "Exception",
      Self::JS_TAG_FLOAT64 => "Float64",
      _ => "Unknown (unexpected)",
    };
    write!(
      f,
      r#"JSValue {{
    u: JSValueUnion,
    tag: {},
  }}"#,
      tag,
    )
  }
}

impl QuickjsRc for JSValue {
  fn free(&mut self, ctx: &mut JSContext) {
    // JS_TAG_MODULE never freed, see quickjs.c#L5518
    if self.tag != Self::JS_TAG_MODULE.into() {
      unsafe { JS_FreeValue_real(ctx, *self) };
    }
  }

  fn dup(&self, ctx: &mut JSContext) -> Self {
    unsafe { JS_DupValue_real(ctx, *self) }
  }
}

impl JSValue {
  pub fn new_object(ctx: &mut JSContext) -> Self {
    unsafe { JS_NewObject(ctx) }
  }

  pub fn new_function(
    ctx: &mut JSContext,
    func: JSFunction,
    name: &str,
    len: i32,
  ) -> Self {
    let name_cstring = CString::new(name).unwrap();
    unsafe {
      JS_NewCFunction_real(
        ctx,
        std::mem::transmute(func as *mut ()),
        name_cstring.as_ptr(),
        len,
      )
    }
  }

  pub fn new_undefined() -> Self {
    Self {
      u: JSValueUnion { int32: 0 },
      tag: Self::JS_TAG_UNDEFINED.into(),
    }
  }

  pub fn to_string_with_len(&self, ctx: &mut JSContext, len: usize) -> String {
    let len = len as *const usize as *mut usize;
    let ptr = unsafe { JS_ToCStringLen_real(ctx, len, *self) };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    unsafe { JS_FreeCString(ctx, ptr) };
    cstr_to_string(cstr)
  }

  pub fn to_string(&self, ctx: &mut JSContext) -> String {
    let ptr = unsafe { JS_ToCString_real(ctx, *self) };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    unsafe { JS_FreeCString(ctx, ptr) };
    cstr_to_string(cstr)
  }

  pub fn is_error(&self, ctx: &mut JSContext) -> bool {
    let jsbool = unsafe { JS_IsError(ctx, *self) };
    jsbool_to_bool(jsbool)
  }

  pub fn is_exception(&self) -> bool {
    unsafe { JS_IsException_real(*self) }
  }

  pub fn is_undefined(&self) -> bool {
    unsafe { JS_IsUndefined_real(*self) }
  }

  pub fn get_property_str<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
    prop: &str,
  ) -> Self {
    let prop_cstring = CString::new(prop).unwrap();
    unsafe { JS_GetPropertyStr(ctx, *self, prop_cstring.as_ptr()) }
  }

  pub fn set_property_str<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
    prop: &str,
    value: Self,
  ) -> Result<bool, JSContextException<'ctx>> {
    let prop_cstring = CString::new(prop).unwrap();
    let result =
      unsafe { JS_SetPropertyStr(ctx, *self, prop_cstring.as_ptr(), value) };
    match result {
      -1 => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
      0 => Ok(false),
      1 => Ok(true),
      _ => panic!("JS_SetPropertyStr return unexpected"),
    }
  }
}

impl<'ctx> JSContextException<'ctx> {
  pub fn from_jsvalue(ctx: &'ctx mut JSContext, value: JSValue) -> Self {
    Self {
      value,
      context: ctx,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::JSRuntime;

  use super::*;

  #[test]
  fn debug_show_js_tag() {
    let val = JSValue::new_undefined();
    assert!(format!("{:?}", val).contains("tag: Undefined"));
  }

  #[test]
  fn new_object() {
    let rt = &mut JSRuntime::new();
    let ctx = &mut JSContext::new(rt);
    let mut val = JSValue::new_object(ctx);
    val.free(ctx);
  }
}
