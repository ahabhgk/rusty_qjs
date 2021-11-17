use std::{
  ffi::CString,
  ops::{Deref, DerefMut},
  ptr::NonNull,
};

use crate::{support::Opaque, JSRuntime, JSValue};

extern "C" {
  fn JS_NewContext(rt: *mut JSRuntime) -> *mut JSContext;
  fn JS_FreeContext(s: *mut JSContext);
  fn JS_DupContext(ctx: *mut JSContext) -> *mut JSContext;
  fn JS_Eval(
    ctx: *mut JSContext,
    input: *const libc::c_char,
    input_len: libc::size_t,
    filename: *const libc::c_char,
    eval_flags: libc::c_int,
  ) -> JSValue;
  fn JS_EvalFunction(ctx: *mut JSContext, fun_obj: JSValue) -> JSValue;
  fn JS_GetException(ctx: *mut JSContext) -> JSValue;
  fn JS_GetGlobalObject(ctx: *mut JSContext) -> JSValue;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)] // Clone?
pub struct JSContext(Opaque);

impl JSContext {
  const JS_EVAL_TYPE_GLOBAL: i32 = (0 << 0);
  const JS_EVAL_TYPE_MODULE: i32 = (1 << 0);
  const JS_EVAL_FLAG_COMPILE_ONLY: i32 = (1 << 5);

  fn eval<'ctx>(
    &'ctx mut self,
    code: &str,
    name: &str,
    is_module: bool,
    compile_only: bool,
  ) -> JSValue {
    let eval_flags = match (is_module, compile_only) {
      (true, true) => {
        Self::JS_EVAL_TYPE_MODULE | Self::JS_EVAL_FLAG_COMPILE_ONLY
      }
      (true, false) => Self::JS_EVAL_TYPE_MODULE,
      (false, true) => {
        Self::JS_EVAL_TYPE_GLOBAL | Self::JS_EVAL_FLAG_COMPILE_ONLY
      }
      (false, false) => Self::JS_EVAL_TYPE_GLOBAL,
    };
    let code_cstring = CString::new(code).unwrap();
    let input = code_cstring.as_ptr();
    let input_len = code.len() as _;
    let name_cstring = CString::new(name).unwrap();
    let filename = name_cstring.as_ptr();

    let evaled =
      unsafe { JS_Eval(self, input, input_len, filename, eval_flags) };
    // Local::from_qjsrc(self, evaled)
    evaled
  }
}

impl JSContext {
  pub fn new(rt: &mut JSRuntime) -> OwnedJSContext {
    let ctx = unsafe { JS_NewContext(rt) };
    let ctx = NonNull::new(ctx).unwrap();
    OwnedJSContext(ctx)
  }

  pub fn eval_module(&mut self, code: &str, name: &str) -> JSValue {
    self.eval(code, name, true, false)
  }

  pub fn compile_module(&mut self, code: &str, name: &str) -> JSValue {
    self.eval(code, name, true, true)
  }

  pub fn eval_script(&mut self, code: &str, name: &str) -> JSValue {
    self.eval(code, name, false, false)
  }

  pub fn compile_script(&mut self, code: &str, name: &str) -> JSValue {
    self.eval(code, name, false, true)
  }

  pub fn eval_function(&mut self, fun_obj: JSValue) -> JSValue {
    // let fun_obj = fun_obj.to_qjsrc();
    let result = unsafe { JS_EvalFunction(self, fun_obj) };
    // Local::from_qjsrc(self, result)
    result
  }

  pub fn get_exception(&mut self) -> JSValue {
    let exception = unsafe { JS_GetException(self) };
    // Local::from_qjsrc(self, exception)
    exception
  }

  pub fn get_global_object(&mut self) -> JSValue {
    let global_object = unsafe { JS_GetGlobalObject(self) };
    // Local::from_qjsrc(self, global_object)
    global_object
  }

  pub fn dup(&mut self) {
    unsafe { JS_DupContext(self) };
  }

  pub fn free(&mut self) {
    unsafe { JS_FreeContext(self) };
  }
}

// impl QuickjsRc for JSContext {
//   fn free(&mut self, ctx: &mut JSContext) {
//     unsafe { JS_FreeContext(self) };
//   }

//   fn dup(&self, ctx: &mut JSContext) -> Self {
//     let ctx = self as *const JSContext as *mut JSContext;
//     let dup = unsafe { JS_DupContext(ctx) };

//   }
// }

pub struct OwnedJSContext(NonNull<JSContext>);

// impl Drop for OwnedJSContext {
//   fn drop(&mut self) {
//     println!("free ctx");
//     self.free();
//   }
// }

impl Deref for OwnedJSContext {
  type Target = JSContext;

  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref() }
  }
}

impl DerefMut for OwnedJSContext {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0.as_mut() }
  }
}
