use std::{
  ffi::{CStr, CString},
  fmt, mem,
};

use crate::{
  error::JSContextException,
  support::{
    cstr_to_string, jsbool_to_bool, MapFnFrom, MapFnTo, ToCFn, UnitType,
  },
  JSContext, QuickjsRc,
};

extern "C" {
  fn JS_FreeValue_real(ctx: *mut JSContext, v: JSValue);
  fn JS_DupValue_real(ctx: *mut JSContext, v: JSValue) -> JSValue;
  fn JS_NewObject(ctx: *mut JSContext) -> JSValue;
  fn JS_NewCFunction_real(
    ctx: *mut JSContext,
    func: *mut GenJSCFunction,
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

type GenJSCFunction = ::std::option::Option<
  unsafe extern "C" fn(
    ctx: *mut JSContext,
    this_val: JSValue,
    argc: libc::c_int,
    argv: *mut JSValue,
  ) -> JSValue,
>;

/// Used by JSValue::new_function to create a JavaScript function from JSFunction.
pub type JSCFunction =
  extern "C" fn(*mut JSContext, JSValue, libc::c_int, *mut JSValue) -> JSValue;

impl<F> MapFnFrom<F> for JSCFunction
where
  F: UnitType + Fn(&mut JSContext, JSValue, &[JSValue]) -> JSValue,
{
  fn mapping() -> Self {
    let f =
      |ctx: *mut JSContext, this: JSValue, argc: i32, argv: *mut JSValue| {
        let mut arguments = Vec::new();
        for i in 0..argc {
          let arg = unsafe { *argv.offset(i as isize) };
          arguments.push(arg);
        }
        let ctx = unsafe { ctx.as_mut() }.unwrap();
        (F::get())(ctx, this, arguments.as_slice())
      };
    f.to_c_fn()
  }
}

#[repr(C)]
#[derive(Copy, Clone)]
union JSValueUnion {
  int32: i32,
  float64: f64,
  ptr: *mut libc::c_void,
  _union_align: u64,
}

/// A QuickJS Value, JSValue represents a Javascript value which
/// can be a primitive type or an object. Reference counting is
/// implemented by QuickjsRc trait.
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
  /// Create a JSValue of object.
  pub fn new_object(ctx: &mut JSContext) -> Self {
    unsafe { JS_NewObject(ctx) }
  }

  /// Create a JSValue of function.
  pub fn new_function<F>(
    ctx: &mut JSContext,
    func: F,
    name: &str,
    len: i32,
  ) -> Self
  where
    F: MapFnTo<JSCFunction>,
  {
    let name_cstring = CString::new(name).unwrap();
    let func = func.map_fn_to();
    unsafe {
      JS_NewCFunction_real(
        ctx,
        mem::transmute(func as *mut ()),
        name_cstring.as_ptr(),
        len,
      )
    }
  }

  /// Create a JSValue of undefined.
  pub fn new_undefined() -> Self {
    Self {
      u: JSValueUnion { int32: 0 },
      tag: Self::JS_TAG_UNDEFINED.into(),
    }
  }

  /// Convert a JSValue to a string with its length.
  /// use JS_ToCStringLen internally.
  pub fn to_string_with_len(&self, ctx: &mut JSContext, len: usize) -> String {
    let len = len as *const usize as *mut usize;
    let ptr = unsafe { JS_ToCStringLen_real(ctx, len, *self) };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    unsafe { JS_FreeCString(ctx, ptr) };
    cstr_to_string(cstr)
  }

  /// Convert a JSValue to a string.
  /// use JS_ToCString internally.
  pub fn to_string(&self, ctx: &mut JSContext) -> String {
    let ptr = unsafe { JS_ToCString_real(ctx, *self) };
    let cstr = unsafe { CStr::from_ptr(ptr) };
    unsafe { JS_FreeCString(ctx, ptr) };
    cstr_to_string(cstr)
  }

  /// Returns true if the JSValue is an error.
  /// use JS_IsError internally.
  pub fn is_error(&self, ctx: &mut JSContext) -> bool {
    let jsbool = unsafe { JS_IsError(ctx, *self) };
    jsbool_to_bool(jsbool)
  }

  /// Returns true if the JSValue is an exception.
  /// use JS_IsException internally.
  pub fn is_exception(&self) -> bool {
    unsafe { JS_IsException_real(*self) }
  }

  /// Returns true if the JSValue is undefined.
  /// use JS_IsUndefined internally.
  pub fn is_undefined(&self) -> bool {
    unsafe { JS_IsUndefined_real(*self) }
  }

  /// Get property from a JSValue by str.
  /// use JS_GetPropertyStr internally.
  pub fn get_property_str<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
    prop: &str,
  ) -> Self {
    let prop_cstring = CString::new(prop).unwrap();
    unsafe { JS_GetPropertyStr(ctx, *self, prop_cstring.as_ptr()) }
  }

  /// Set property on a JSValue by str.
  /// use JS_SetPropertyStr internally.
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
  /// Create a JSContextException from a JSValue and its JSContext.
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
