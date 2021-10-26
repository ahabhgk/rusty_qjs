use super::{error::JsError, runtime::JsRuntime, value::JsValue};
use std::{ffi::CString, ptr::NonNull};

#[derive(Debug)]
pub struct JsContext(pub NonNull<libquickjs_sys::JSContext>);

impl Drop for JsContext {
  fn drop(&mut self) {
    unsafe { libquickjs_sys::JS_FreeContext(self.0.as_mut()) };
  }
}

impl JsContext {
  pub fn new(runtime: &mut JsRuntime) -> Self {
    let rt = unsafe { runtime.0.as_mut() };
    let ctx = unsafe { libquickjs_sys::JS_NewContext(rt) };
    let ctx = NonNull::new(ctx).unwrap();
    Self(ctx)
  }

  /// is_module: module or global
  pub fn eval(
    &mut self,
    code: &str,
    name: &str,
    is_module: bool,
    compile_only: bool,
  ) -> Result<JsValue, JsError> {
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

    let value = unsafe {
      libquickjs_sys::JS_Eval(
        self.0.as_mut(),
        input,
        input_len,
        filename,
        eval_flags,
      )
    };
    let value = JsValue::new(self, value);

    if value.is_exception() {
      return Err(JsError::dump_from_context(self));
    }
    Ok(value)
  }

  pub fn eval_function(&mut self, func_obj: &JsValue) -> JsValue {
    let value =
      unsafe { libquickjs_sys::JS_EvalFunction(self.0.as_mut(), func_obj.val) };
    JsValue::new(self, value)
  }
}
