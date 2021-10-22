use std::ffi::CString;

use crate::{
  context::{JSContext, JsContext},
  runtime::{JSRuntime, JsRuntime},
  value::JSValue,
};

extern "C" {
  fn JS_NewAtomLen(
    ctx: *mut JSContext,
    str_: *const libc::c_char,
    len: libc::size_t,
  ) -> JSAtom;
  fn JS_NewAtom(ctx: *mut JSContext, str_: *const libc::c_char) -> JSAtom;
  fn JS_NewAtomUInt32(ctx: *mut JSContext, n: u32) -> JSAtom;
  fn JS_DupAtom(ctx: *mut JSContext, v: JSAtom) -> JSAtom;
  fn JS_FreeAtom(ctx: *mut JSContext, v: JSAtom);
  fn JS_FreeAtomRT(rt: *mut JSRuntime, v: JSAtom);
  fn JS_AtomToValue(ctx: *mut JSContext, atom: JSAtom) -> JSValue;
  fn JS_AtomToString(ctx: *mut JSContext, atom: JSAtom) -> JSValue;
  fn JS_AtomToCString(ctx: *mut JSContext, atom: JSAtom)
    -> *const libc::c_char;
  fn JS_ValueToAtom(ctx: *mut JSContext, val: JSValue) -> JSAtom;
}

pub type JSAtom = u32;

pub struct JsAtom(JSAtom);

impl JsAtom {
  pub fn new(ctx: &mut JsContext, str: &str) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let str = CString::new(str).unwrap();
    let atom = unsafe { JS_NewAtom(ctx, str.as_ptr()) };
    Self(atom)
  }

  pub fn new_len(ctx: &mut JsContext, str: &str, len: usize) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let str = CString::new(str).unwrap();
    let atom = unsafe { JS_NewAtomLen(ctx, str.as_ptr(), len) };
    Self(atom)
  }

  pub fn new_uint32(ctx: &mut JsContext, n: u32) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let atom = unsafe { JS_NewAtomUInt32(ctx, n) };
    Self(atom)
  }

  pub fn dup(&self, ctx: &mut JsContext) -> Self {
    let ctx = unsafe { ctx.0.as_mut() };
    let atom = unsafe { JS_DupAtom(ctx, self.0) };
    Self(atom)
  }

  pub fn free(&self, ctx: &mut JsContext) {
    let ctx = unsafe { ctx.0.as_mut() };
    unsafe { JS_FreeAtom(ctx, self.0) };
  }

  pub fn free_on_runtime(&self, rt: &mut JsRuntime) {
    let rt = unsafe { rt.0.as_mut() };
    unsafe { JS_FreeAtomRT(rt, self.0) };
  }
}
