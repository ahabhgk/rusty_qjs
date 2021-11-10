use std::{ffi::CString, ptr::NonNull};

use crate::{runtime::JsRuntime, value::JsValue};

#[derive(Debug)]
pub struct JsContext(pub NonNull<libquickjs_sys::JSContext>);

impl JsContext {
  pub fn new(runtime: &mut JsRuntime) -> Self {
    let ctx = unsafe { libquickjs_sys::JS_NewContext(runtime.0.as_mut()) };
    Self::from_raw(ctx)
  }

  pub fn from_raw(raw_context: *mut libquickjs_sys::JSContext) -> Self {
    let context = NonNull::new(raw_context).unwrap();
    Self(context)
  }

  pub fn free(&mut self) {
    unsafe { libquickjs_sys::JS_FreeContext(self.0.as_mut()) };
  }

  pub fn eval_module(&mut self, code: &str, name: &str) -> JsValue {
    self.eval(code, name, true, false)
  }

  pub fn compile_module(&mut self, code: &str, name: &str) -> JsValue {
    self.eval(code, name, true, true)
  }

  pub fn eval_script(&mut self, code: &str, name: &str) -> JsValue {
    self.eval(code, name, false, false)
  }

  pub fn compile_script(&mut self, code: &str, name: &str) -> JsValue {
    self.eval(code, name, false, true)
  }

  fn eval(
    &mut self,
    code: &str,
    name: &str,
    is_module: bool,
    compile_only: bool,
  ) -> JsValue {
    let eval_flags = match (is_module, compile_only) {
      (true, true) => {
        libquickjs_sys::JS_EVAL_TYPE_MODULE
          | libquickjs_sys::JS_EVAL_FLAG_COMPILE_ONLY
      }
      (true, false) => libquickjs_sys::JS_EVAL_TYPE_MODULE,
      (false, true) => {
        libquickjs_sys::JS_EVAL_TYPE_GLOBAL
          | libquickjs_sys::JS_EVAL_FLAG_COMPILE_ONLY
      }
      (false, false) => libquickjs_sys::JS_EVAL_TYPE_GLOBAL,
    } as _;
    let code_cstring = CString::new(code).unwrap();
    let input = code_cstring.as_ptr();
    let input_len = code.len() as _;
    let name_cstring = CString::new(name).unwrap();
    let filename = name_cstring.as_ptr();

    let raw_context = unsafe { self.0.as_mut() };
    let value = unsafe {
      libquickjs_sys::JS_Eval(
        raw_context,
        input,
        input_len,
        filename,
        eval_flags,
      )
    };
    JsValue::from_raw(raw_context, value)
  }

  pub fn eval_function(&mut self, func_obj: &JsValue) -> JsValue {
    let raw_context = unsafe { self.0.as_mut() };
    let value = unsafe {
      libquickjs_sys::JS_EvalFunction(raw_context, func_obj.raw_value)
    };
    JsValue::from_raw(raw_context, value)
  }

  pub fn get_exception(&mut self) -> JsValue {
    let raw_context = unsafe { self.0.as_mut() };
    let exception = unsafe { libquickjs_sys::JS_GetException(raw_context) };
    JsValue::from_raw(raw_context, exception)
  }

  pub fn get_global_object(&mut self) -> JsValue {
    let raw_context = unsafe { self.0.as_mut() };
    let global_object =
      unsafe { libquickjs_sys::JS_GetGlobalObject(raw_context) };
    JsValue::from_raw(raw_context, global_object)
  }
}
