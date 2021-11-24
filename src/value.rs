use std::{
  ffi::{CStr, CString},
  fmt, mem, ptr,
};

use crate::{
  error::JSContextException,
  support::{cstr_to_string, MapFnFrom, MapFnTo, ToCFn, UnitType},
  JSContext, JSRuntime, QuickjsRc,
};

extern "C" {
  fn JS_FreeValue_real(ctx: *mut JSContext, v: JSValue);
  fn JS_FreeValueRT_real(rt: *mut JSRuntime, v: JSValue);
  fn JS_DupValue_real(ctx: *mut JSContext, v: JSValue) -> JSValue;
  fn JS_DupValueRT_real(ctx: *mut JSRuntime, v: JSValue) -> JSValue;
  fn JS_NewBool_real(ctx: *mut JSContext, v: bool) -> JSValue;
  fn JS_NewInt32_real(ctx: *mut JSContext, v: i32) -> JSValue;
  fn JS_NewInt64_real(ctx: *mut JSContext, v: i64) -> JSValue;
  fn JS_NewUint32_real(ctx: *mut JSContext, v: u32) -> JSValue;
  fn JS_NewFloat64_real(ctx: *mut JSContext, v: f64) -> JSValue;
  pub fn JS_NewBigInt64(ctx: *mut JSContext, v: i64) -> JSValue;
  pub fn JS_NewBigUint64(ctx: *mut JSContext, v: u64) -> JSValue;
  fn JS_NewCatchOffset_real(ctx: *mut JSContext, v: i32) -> JSValue;
  fn JS_NewObject(ctx: *mut JSContext) -> JSValue;
  fn JS_NewCFunction_real(
    ctx: *mut JSContext,
    func: *mut GenJSCFunction,
    name: *const libc::c_char,
    length: libc::c_int,
  ) -> JSValue;
  fn JS_ToCStringLen2(
    ctx: *mut JSContext,
    plen: *mut libc::size_t,
    val1: JSValue,
    cesu8: libc::c_int,
  ) -> *const libc::c_char;
  fn JS_FreeCString(ctx: *mut JSContext, ptr: *const libc::c_char);
  fn JS_IsNumber_real(v: JSValue) -> bool;
  fn JS_IsBigInt_real(ctx: *mut JSContext, v: JSValue) -> bool;
  fn JS_IsBigFloat_real(v: JSValue) -> bool;
  fn JS_IsBigDecimal_real(v: JSValue) -> bool;
  fn JS_IsBool_real(v: JSValue) -> bool;
  fn JS_IsNull_real(v: JSValue) -> bool;
  fn JS_IsException_real(v: JSValue) -> bool;
  fn JS_IsUndefined_real(v: JSValue) -> bool;
  fn JS_IsUninitialized_real(v: JSValue) -> bool;
  fn JS_IsString_real(v: JSValue) -> bool;
  fn JS_IsSymbol_real(v: JSValue) -> bool;
  fn JS_IsObject_real(v: JSValue) -> bool;
  fn JS_NewError(ctx: *mut JSContext) -> JSValue;
  fn JS_Throw(ctx: *mut JSContext, obj: JSValue) -> JSValue;
  fn JS_IsError(ctx: *mut JSContext, val: JSValue) -> bool;
  fn JS_ThrowSyntaxError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  fn JS_ThrowTypeError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  fn JS_ThrowReferenceError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  fn JS_ThrowRangeError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  fn JS_ThrowInternalError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  fn JS_ThrowOutOfMemory(ctx: *mut JSContext) -> JSValue;
  fn JS_ToBool(ctx: *mut JSContext, val: JSValue) -> libc::c_int;
  fn JS_ToInt32(
    ctx: *mut JSContext,
    pres: *mut i32,
    val: JSValue,
  ) -> libc::c_int;
  fn JS_ToUint32_real(
    ctx: *mut JSContext,
    pres: *mut u32,
    val: JSValue,
  ) -> libc::c_int;
  fn JS_ToInt64(
    ctx: *mut JSContext,
    pres: *mut i64,
    val: JSValue,
  ) -> libc::c_int;
  fn JS_ToIndex(
    ctx: *mut JSContext,
    plen: *mut u64,
    val: JSValue,
  ) -> libc::c_int;
  fn JS_ToFloat64(
    ctx: *mut JSContext,
    pres: *mut f64,
    val: JSValue,
  ) -> libc::c_int;
  fn JS_ToBigInt64(
    ctx: *mut JSContext,
    pres: *mut i64,
    val: JSValue,
  ) -> libc::c_int;
  fn JS_ToInt64Ext(
    ctx: *mut JSContext,
    pres: *mut i64,
    val: JSValue,
  ) -> libc::c_int;
  fn JS_NewStringLen(
    ctx: *mut JSContext,
    str1: *const libc::c_char,
    len1: libc::size_t,
  ) -> JSValue;
  fn JS_NewString(ctx: *mut JSContext, str_: *const libc::c_char) -> JSValue;
  fn JS_NewAtomString(ctx: *mut JSContext, str: *const libc::c_char)
    -> JSValue;
  fn JS_ToString(ctx: *mut JSContext, val: JSValue) -> JSValue;
  fn JS_ToPropertyKey(ctx: *mut JSContext, val: JSValue) -> JSValue;
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

  fn free_runtime(&mut self, rt: &mut JSRuntime) {
    // JS_TAG_MODULE never freed, see quickjs.c#L5518
    if self.tag != Self::JS_TAG_MODULE.into() {
      unsafe { JS_FreeValueRT_real(rt, *self) };
    }
  }

  fn dup(&mut self, ctx: &mut JSContext) -> Self {
    unsafe { JS_DupValue_real(ctx, *self) }
  }

  fn dup_runtime(&mut self, rt: &mut JSRuntime) -> Self {
    unsafe { JS_DupValueRT_real(rt, *self) }
  }
}

impl JSValue {
  /// Create a JSValue of boolean. use JS_NewBool internally.
  pub fn new_bool(ctx: &mut JSContext, value: bool) -> Self {
    unsafe { JS_NewBool_real(ctx, value) }
  }

  /// Create a JSValue of int32. use JS_NewInt32 internally.
  pub fn new_int32(ctx: &mut JSContext, value: i32) -> Self {
    unsafe { JS_NewInt32_real(ctx, value) }
  }

  /// Create a JSValue of int64. use JS_NewInt64 internally.
  pub fn new_int64(ctx: &mut JSContext, value: i64) -> Self {
    unsafe { JS_NewInt64_real(ctx, value) }
  }

  /// Create a JSValue of uint32. use JS_NewUint32 internally.
  pub fn new_uint32(ctx: &mut JSContext, value: u32) -> Self {
    unsafe { JS_NewUint32_real(ctx, value) }
  }

  /// Create a JSValue of float64. use JS_NewFloat64 internally.
  pub fn new_float64(ctx: &mut JSContext, value: f64) -> Self {
    unsafe { JS_NewFloat64_real(ctx, value) }
  }

  /// Create a JSValue of big int64. use JS_NewBigInt64 internally.
  pub fn new_big_int64(ctx: &mut JSContext, value: i64) -> Self {
    unsafe { JS_NewBigInt64(ctx, value) }
  }

  /// Create a JSValue of big uint64. use JS_NewBigUint64 internally.
  pub fn new_big_uint64(ctx: &mut JSContext, value: u64) -> Self {
    unsafe { JS_NewBigUint64(ctx, value) }
  }

  /// Create a JSValue of object. use JS_NewObject internally.
  pub fn new_object(ctx: &mut JSContext) -> Self {
    unsafe { JS_NewObject(ctx) }
  }

  /// Create a JSValue of function. use JS_NewCFunction internally.
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

  /// Create a JSValue of Error.
  pub fn new_error(ctx: &mut JSContext) -> Self {
    unsafe { JS_NewError(ctx) }
  }

  /// Create a JSValue of catch offset. use JS_NewCatchOffset internally.
  pub fn new_catch_offset(ctx: &mut JSContext, value: i32) -> Self {
    unsafe { JS_NewCatchOffset_real(ctx, value) }
  }

  /// Convert a JSValue to a rust string with its length and cesu8, cesu8 determines
  /// if non-BMP1 codepoints are encoded as 1 or 2 utf-8 sequences. use JS_ToCStringLen2
  /// internally.
  pub fn to_rust_string_with_length_and_cesu8<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
    len: usize,
    cesu8: bool,
  ) -> Result<String, JSContextException<'ctx>> {
    let len = len as *const usize as *mut _;
    let cesu8 = if cesu8 { 1 } else { 0 };
    let p = unsafe { JS_ToCStringLen2(ctx, len, *self, cesu8) };
    if p.is_null() {
      let e = ctx.get_exception();
      Err(JSContextException::from_jsvalue(ctx, e))
    } else {
      let cstr = unsafe { CStr::from_ptr(p) };
      unsafe { JS_FreeCString(ctx, p) };
      Ok(cstr_to_string(cstr))
    }
  }

  /// Convert a JSValue to a rust string with its length. use JS_ToCStringLen internally.
  pub fn to_rust_string_with_length<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
    len: usize,
  ) -> Result<String, JSContextException<'ctx>> {
    self.to_rust_string_with_length_and_cesu8(ctx, len, false)
  }

  /// Convert a JSValue to a rust string. use JS_ToCString internally.
  pub fn to_rust_string<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<String, JSContextException<'ctx>> {
    self.to_rust_string_with_length_and_cesu8(ctx, 0, false)
  }

  /// Returns true if the JSValue is a number. use JS_IsNumber internally.
  pub fn is_number(&self) -> bool {
    unsafe { JS_IsNumber_real(*self) }
  }

  /// Returns true if the JSValue is a big int. use JS_IsBigInt internally.
  pub fn is_big_int(&self, ctx: &mut JSContext) -> bool {
    unsafe { JS_IsBigInt_real(ctx, *self) }
  }

  /// Returns true if the JSValue is a big float. use JS_IsBigFloat internally.
  pub fn is_big_float(&self) -> bool {
    unsafe { JS_IsBigFloat_real(*self) }
  }

  /// Returns true if the JSValue is a big decimal. use JS_IsBigDecimal internally.
  pub fn is_big_decimal(&self) -> bool {
    unsafe { JS_IsBigDecimal_real(*self) }
  }

  /// Returns true if the JSValue is a boolean. use JS_IsBool internally.
  pub fn is_bool(&self) -> bool {
    unsafe { JS_IsBool_real(*self) }
  }

  /// Returns true if the JSValue is null. use JS_IsNull internally.
  pub fn is_null(&self) -> bool {
    unsafe { JS_IsNull_real(*self) }
  }

  /// Returns true if the JSValue is an error. use JS_IsError internally.
  pub fn is_error(&self, ctx: &mut JSContext) -> bool {
    unsafe { JS_IsError(ctx, *self) }
  }

  /// Returns true if the JSValue is an exception. use JS_IsException internally.
  pub fn is_exception(&self) -> bool {
    unsafe { JS_IsException_real(*self) }
  }

  /// Returns true if the JSValue is undefined. use JS_IsUndefined internally.
  pub fn is_undefined(&self) -> bool {
    unsafe { JS_IsUndefined_real(*self) }
  }

  /// Returns true if the JSValue is uninitialized. use JS_IsUninitialized internally.
  pub fn is_uninitialized(&self) -> bool {
    unsafe { JS_IsUninitialized_real(*self) }
  }

  /// Returns true if the JSValue is a string. use JS_IsString internally.
  pub fn is_string(&self) -> bool {
    unsafe { JS_IsString_real(*self) }
  }

  /// Returns true if the JSValue is a symbol. use JS_IsSymbol internally.
  pub fn is_symbol(&self) -> bool {
    unsafe { JS_IsSymbol_real(*self) }
  }

  /// Returns true if the JSValue is an object. use JS_IsObject internally.
  pub fn is_object(&self) -> bool {
    unsafe { JS_IsObject_real(*self) }
  }

  /// Throw the JSValue, `JSValue::new_bool(ctx, false).throw(ctx)` => `throw false`.
  /// use JS_Throw internally.
  pub fn throw(&self, ctx: &mut JSContext) -> Self {
    unsafe { JS_Throw(ctx, *self) }
  }

  /// Create a SyntaxError with message and throw it, use JS_ThrowSyntaxError internally.
  pub fn throw_syntax_error(ctx: &mut JSContext, message: &str) -> Self {
    let message = CString::new(message).unwrap();
    unsafe { JS_ThrowSyntaxError(ctx, message.as_ptr()) }
  }

  /// Create a TypeError with message and throw it, use JS_ThrowTypeError internally.
  pub fn throw_type_error(ctx: &mut JSContext, message: &str) -> Self {
    let message = CString::new(message).unwrap();
    unsafe { JS_ThrowTypeError(ctx, message.as_ptr()) }
  }

  /// Create a TypeError with message and throw it, use JS_ThrowReferenceError internally.
  pub fn throw_reference_error(ctx: &mut JSContext, message: &str) -> Self {
    let message = CString::new(message).unwrap();
    unsafe { JS_ThrowReferenceError(ctx, message.as_ptr()) }
  }

  /// Create a TypeError with message and throw it, use JS_ThrowRangeError internally.
  pub fn throw_range_error(ctx: &mut JSContext, message: &str) -> Self {
    let message = CString::new(message).unwrap();
    unsafe { JS_ThrowRangeError(ctx, message.as_ptr()) }
  }

  /// Create a TypeError with message and throw it, use JS_ThrowInternalError internally.
  pub fn throw_internal_error(ctx: &mut JSContext, message: &str) -> Self {
    let message = CString::new(message).unwrap();
    unsafe { JS_ThrowInternalError(ctx, message.as_ptr()) }
  }

  /// Create a TypeError with message and throw it, use JS_ThrowOutOfMemory internally.
  pub fn throw_out_of_memory(ctx: &mut JSContext) -> Self {
    unsafe { JS_ThrowOutOfMemory(ctx) }
  }

  /// Convert a JSValue to a bool, use JS_ToBool internally.
  pub fn to_bool<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<bool, JSContextException<'ctx>> {
    let res = unsafe { JS_ToBool(ctx, *self) };
    match res {
      -1 => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
      0 => Ok(false),
      _ => Ok(true),
    }
  }

  /// Convert a JSValue to an i32, use JS_ToInt32 internally.
  pub fn to_int32<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<i32, JSContextException<'ctx>> {
    let pres = ptr::null_mut();
    let res = unsafe { JS_ToInt32(ctx, pres, *self) };
    match res {
      0 => Ok(unsafe { *pres }),
      _ => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
    }
  }

  /// Convert a JSValue to an u32, use JS_ToUint32 internally.
  pub fn to_uint32<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<u32, JSContextException<'ctx>> {
    let pres = ptr::null_mut();
    let res = unsafe { JS_ToUint32_real(ctx, pres, *self) };
    match res {
      0 => Ok(unsafe { *pres }),
      _ => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
    }
  }

  /// Convert a JSValue to an i64, use JS_ToInt64 internally.
  pub fn to_int64<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<i64, JSContextException<'ctx>> {
    let pres = ptr::null_mut();
    let res = unsafe { JS_ToInt64(ctx, pres, *self) };
    match res {
      0 => Ok(unsafe { *pres }),
      _ => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
    }
  }

  /// Convert a JSValue to an array index, use JS_ToIndex internally.
  pub fn to_index<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<u64, JSContextException<'ctx>> {
    let plen = ptr::null_mut();
    let res = unsafe { JS_ToIndex(ctx, plen, *self) };
    match res {
      0 => Ok(unsafe { *plen }),
      _ => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
    }
  }

  /// Convert a JSValue to a float64, use JS_ToFloat64 internally.
  pub fn to_float64<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<f64, JSContextException<'ctx>> {
    let pres = ptr::null_mut();
    let res = unsafe { JS_ToFloat64(ctx, pres, *self) };
    match res {
      0 => Ok(unsafe { *pres }),
      _ => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
    }
  }

  /// Convert a JSValue to a big int64, return an exception if the JSValue
  /// is a Number. use JS_ToBigInt64 internally.
  pub fn to_big_int64<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<i64, JSContextException<'ctx>> {
    let pres = ptr::null_mut();
    let res = unsafe { JS_ToBigInt64(ctx, pres, *self) };
    match res {
      0 => Ok(unsafe { *pres }),
      _ => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
    }
  }

  /// same as JSValue::to_int64 but allow BigInt. use JS_ToInt64Ext internally.
  pub fn to_int64_ext<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
  ) -> Result<i64, JSContextException<'ctx>> {
    let pres = ptr::null_mut();
    let res = unsafe { JS_ToInt64Ext(ctx, pres, *self) };
    match res {
      0 => Ok(unsafe { *pres }),
      _ => {
        let e = ctx.get_exception();
        Err(JSContextException::from_jsvalue(ctx, e))
      }
    }
  }

  /// Create a JSValue of string with its length. use JS_NewStringLen internally.
  pub fn new_string_with_length(
    ctx: &mut JSContext,
    value: &str,
    len: usize,
  ) -> Self {
    let value = CString::new(value).unwrap();
    unsafe { JS_NewStringLen(ctx, value.as_ptr(), len) }
  }

  /// Create a JSValue of string. use JS_NewString internally.
  pub fn new_string(ctx: &mut JSContext, value: &str) -> Self {
    let value = CString::new(value).unwrap();
    unsafe { JS_NewString(ctx, value.as_ptr()) }
  }

  /// Create a JSValue of string. use JS_NewAtomString internally.
  pub fn new_atom_string(ctx: &mut JSContext, value: &str) -> Self {
    let value = CString::new(value).unwrap();
    unsafe { JS_NewAtomString(ctx, value.as_ptr()) }
  }

  /// Convert a JSValue to a js string. use JS_ToString internally.
  pub fn to_string(&self, ctx: &mut JSContext) -> Self {
    unsafe { JS_ToString(ctx, *self) }
  }

  /// Convert a JSValue to a property key (string or symbol). use JS_ToPropertyKey
  /// internally.
  pub fn to_property_key(&self, ctx: &mut JSContext) -> Self {
    unsafe { JS_ToPropertyKey(ctx, *self) }
  }

  /// Get property from a JSValue by a &str prop. use JS_GetPropertyStr internally.
  pub fn get_property_str<'ctx>(
    &self,
    ctx: &'ctx mut JSContext,
    prop: &str,
  ) -> Self {
    let prop_cstring = CString::new(prop).unwrap();
    unsafe { JS_GetPropertyStr(ctx, *self, prop_cstring.as_ptr()) }
  }

  /// Set property on a JSValue by a &str prop. use JS_SetPropertyStr internally.
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
