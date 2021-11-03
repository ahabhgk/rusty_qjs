use std::ffi::CString;

use crate::{context::JsContext, runtime::JsRuntime};

pub struct JsAtom(libquickjs_sys::JSAtom);

impl JsAtom {
  pub fn new(ctx: &mut JsContext, str: &str) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let str = CString::new(str).unwrap();
    let atom = unsafe { libquickjs_sys::JS_NewAtom(ctx, str.as_ptr()) };
    Self(atom)
  }

  pub fn new_len(ctx: &mut JsContext, str: &str, len: u64) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let str = CString::new(str).unwrap();
    let atom = unsafe { libquickjs_sys::JS_NewAtomLen(ctx, str.as_ptr(), len) };
    Self(atom)
  }

  pub fn new_uint32(ctx: &mut JsContext, n: u32) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let atom = unsafe { libquickjs_sys::JS_NewAtomUInt32(ctx, n) };
    Self(atom)
  }

  pub fn dup(&self, ctx: &mut JsContext) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let atom = unsafe { libquickjs_sys::JS_DupAtom(ctx, self.0) };
    Self(atom)
  }

  pub fn free(&self, ctx: &mut JsContext) {
    let ctx = unsafe { ctx.0.as_mut() };
    unsafe { libquickjs_sys::JS_FreeAtom(ctx, self.0) };
  }

  pub fn free_runtime(&self, rt: &mut JsRuntime) {
    let rt = unsafe { rt.0.as_mut() };
    unsafe { libquickjs_sys::JS_FreeAtomRT(rt, self.0) };
  }
}
