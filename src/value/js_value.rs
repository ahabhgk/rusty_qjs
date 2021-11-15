use std::fmt;

use crate::{context::JSContext, Local, QuickjsRc};

extern "C" {
  fn JS_FreeValue_real(ctx: *mut JSContext, v: JSValue);
  fn JS_DupValue_real(ctx: *mut JSContext, v: JSValue) -> JSValue;
}

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
  tag: {:?},
}}"#,
      tag,
    )
  }
}

impl QuickjsRc for JSValue {
  fn free(&mut self, ctx: &mut JSContext) {
    // JS_TAG_MODULE never freed, see quickjs.c#L5518
    if self.tag == Self::JS_TAG_MODULE.into() {
      return;
    }
    unsafe { JS_FreeValue_real(ctx, *self) };
  }

  fn dup(&self, ctx: &mut JSContext) -> Self {
    unsafe { JS_DupValue_real(ctx, *self) }
  }
}

impl JSValue {
  pub fn new_undefined<'ctx>(ctx: &'ctx mut JSContext) -> Local<'ctx, Self> {
    let rc = Self {
      u: JSValueUnion { int32: 0 },
      tag: Self::JS_TAG_UNDEFINED.into(),
    };
    Local::new(ctx, rc)
  }
}

#[cfg(test)]
mod tests {
  use crate::runtime::JSRuntime;

  use super::*;

  #[test]
  fn debug_show_js_tag() {
    let mut rt = JSRuntime::new();
    let mut ctx = JSContext::new(&mut rt);
    let val = JSValue::new_undefined(&mut ctx);
    assert!(format!("{:?}", val).contains("tag: \"Undefined\""));
  }
}
