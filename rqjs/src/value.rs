use std::ffi;

use crate::{
  context::{JSContext, JsContext},
  runtime::JSRuntime,
};

extern "C" {
  pub fn JS_Throw(ctx: *mut JSContext, obj: JSValue) -> JSValue;
  pub fn JS_GetException(ctx: *mut JSContext) -> JSValue;
  pub fn JS_IsError(ctx: *mut JSContext, val: JSValue)
    -> ::std::os::raw::c_int;
  pub fn JS_NewError(ctx: *mut JSContext) -> JSValue;
  pub fn JS_ThrowSyntaxError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  pub fn JS_ThrowTypeError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  pub fn JS_ThrowReferenceError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  pub fn JS_ThrowRangeError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  pub fn JS_ThrowInternalError(
    ctx: *mut JSContext,
    fmt: *const libc::c_char,
    ...
  ) -> JSValue;
  pub fn JS_ThrowOutOfMemory(ctx: *mut JSContext) -> JSValue;
  pub fn __JS_FreeValue(ctx: *mut JSContext, v: JSValue);
  pub fn __JS_FreeValueRT(rt: *mut JSRuntime, v: JSValue);
  pub fn JS_ToBool(ctx: *mut JSContext, val: JSValue) -> ::std::os::raw::c_int;
  pub fn JS_ToInt32(
    ctx: *mut JSContext,
    pres: *mut i32,
    val: JSValue,
  ) -> ::std::os::raw::c_int;
  pub fn JS_ToInt64(
    ctx: *mut JSContext,
    pres: *mut i64,
    val: JSValue,
  ) -> ::std::os::raw::c_int;
  pub fn JS_ToIndex(
    ctx: *mut JSContext,
    plen: *mut u64,
    val: JSValue,
  ) -> ::std::os::raw::c_int;
  pub fn JS_ToFloat64(
    ctx: *mut JSContext,
    pres: *mut f64,
    val: JSValue,
  ) -> ::std::os::raw::c_int;
  pub fn JS_ToBigInt64(
    ctx: *mut JSContext,
    pres: *mut i64,
    val: JSValue,
  ) -> ::std::os::raw::c_int;
  pub fn JS_ToInt64Ext(
    ctx: *mut JSContext,
    pres: *mut i64,
    val: JSValue,
  ) -> ::std::os::raw::c_int;
  pub fn JS_ToString(ctx: *mut JSContext, val: JSValue) -> JSValue;
  pub fn JS_ToPropertyKey(ctx: *mut JSContext, val: JSValue) -> JSValue;

  pub fn JS_NewBigInt64(ctx: *mut JSContext, v: i64) -> JSValue;
  pub fn JS_NewBigUint64(ctx: *mut JSContext, v: u64) -> JSValue;
  pub fn JS_NewStringLen(
    ctx: *mut JSContext,
    str1: *const libc::c_char,
    len1: libc::size_t,
  ) -> JSValue;
  pub fn JS_NewString(
    ctx: *mut JSContext,
    str_: *const libc::c_char,
  ) -> JSValue;
  pub fn JS_NewAtomString(
    ctx: *mut JSContext,
    str_: *const libc::c_char,
  ) -> JSValue;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union JSValueUnion {
  pub int32: i32,
  pub float64: f64,
  pub ptr: *mut ffi::c_void,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct JSValue {
  pub u: JSValueUnion,
  pub tag: i64,
}

pub struct JsValue(pub JSValue);

impl JsValue {
  pub fn new_big_int64(ctx: &mut JsContext, v: i64) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let value = unsafe { JS_NewBigInt64(ctx, v) };
    Self(value)
  }

  pub fn new_big_uint64(ctx: &mut JsContext, v: u64) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let value = unsafe { JS_NewBigUint64(ctx, v) };
    Self(value)
  }
}
