use std::{
  ffi::CString,
  marker::PhantomData,
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
  fn JS_GetRuntime(ctx: *mut JSContext) -> *mut JSRuntime;
}

/// JSContext represents a Javascript context (or Realm).
/// Each JSContext has its own global objects and system objects.
/// There can be several JSContexts per JSRuntime and they can share objects,
/// similar to frames of the same origin sharing Javascript objects in a web browser.
#[repr(C)]
#[derive(Debug, Copy, Clone)] // Clone?
pub struct JSContext(Opaque);

impl JSContext {
  #[allow(clippy::identity_op)]
  const JS_EVAL_TYPE_GLOBAL: i32 = (0 << 0);
  const JS_EVAL_TYPE_MODULE: i32 = (1 << 0);
  const JS_EVAL_FLAG_COMPILE_ONLY: i32 = (1 << 5);

  fn eval(
    &mut self,
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

    unsafe { JS_Eval(self, input, input_len, filename, eval_flags) }
  }
}

impl JSContext {
  /// Create a new JSContext. use JS_NewContext internally.
  #[allow(clippy::new_ret_no_self)]
  pub fn new(rt: &mut JSRuntime) -> OwnedJSContext {
    let ctx = unsafe { JS_NewContext(rt) };
    let ctx = NonNull::new(ctx).unwrap();
    OwnedJSContext(ctx, PhantomData)
  }

  /// Evaluate the module on this JSContext. use JS_Eval internally.
  pub fn eval_module(&mut self, code: &str, name: &str) -> JSValue {
    self.eval(code, name, true, false)
  }

  /// Compile the module on this JSContext. use JS_Eval internally.
  pub fn compile_module(&mut self, code: &str, name: &str) -> JSValue {
    self.eval(code, name, true, true)
  }

  /// Evaluate the script on this JSContext. use JS_Eval internally.
  pub fn eval_script(&mut self, code: &str, name: &str) -> JSValue {
    self.eval(code, name, false, false)
  }

  /// Compile the script on this JSContext. use JS_Eval internally.
  pub fn compile_script(&mut self, code: &str, name: &str) -> JSValue {
    self.eval(code, name, false, true)
  }

  /// Evaluate the function on this JSContext. use JS_EvalFunction internally.
  pub fn eval_function(&mut self, fun_obj: JSValue) -> JSValue {
    unsafe { JS_EvalFunction(self, fun_obj) }
  }

  /// Get the exception of this JSContext. use JS_GetException internally.
  pub fn get_exception(&mut self) -> JSValue {
    unsafe { JS_GetException(self) }
  }

  /// Get the global object of this JSContext. use JS_GetGlobalObject internally.
  pub fn get_global_object(&mut self) -> JSValue {
    unsafe { JS_GetGlobalObject(self) }
  }

  /// Get the JSRuntime of this JSContext. use JS_GetRuntime internally.
  pub fn get_runtime(&mut self) -> &mut JSRuntime {
    let rt = unsafe { JS_GetRuntime(self) };
    let rt = unsafe { rt.as_mut() }.unwrap();
    rt
  }

  /// Duplicate the JSContext reference count. use JS_DupContext internally.
  pub fn dup(&mut self) -> *mut Self {
    unsafe { JS_DupContext(self) }
  }

  /// Free the JSContext. use JS_FreeContext internally.
  pub fn free(&mut self) {
    unsafe { JS_FreeContext(self) };
  }
}

/// Same as JSContext but gets freed when it drops.
pub struct OwnedJSContext<'rt>(NonNull<JSContext>, PhantomData<&'rt JSRuntime>);

impl Drop for OwnedJSContext<'_> {
  fn drop(&mut self) {
    self.free();
  }
}

impl Deref for OwnedJSContext<'_> {
  type Target = JSContext;

  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref() }
  }
}

impl DerefMut for OwnedJSContext<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0.as_mut() }
  }
}
