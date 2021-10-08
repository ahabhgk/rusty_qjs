use super::{error::JsError, runtime::JsRuntime, value::JsValue};
use libquickjs_sys as qjs;
use std::{ffi::CString, marker::PhantomData, rc::Rc};

#[derive(Debug)]
pub struct JsContext {
  inner: *mut qjs::JSContext,
  _marker: PhantomData<*mut qjs::JSContext>,
}

impl Drop for JsContext {
  fn drop(&mut self) {
    unsafe { qjs::JS_FreeContext(self.inner) };
  }
}

impl Default for JsContext {
  fn default() -> Self {
    let runtime = JsRuntime::default();
    let context = unsafe { qjs::JS_NewContext(runtime.inner()) };
    Self {
      inner: context,
      _marker: PhantomData,
    }
  }
}

impl JsContext {
  pub fn new(runtime: &JsRuntime) -> Rc<Self> {
    let context = unsafe { qjs::JS_NewContext(runtime.inner()) };
    let context = Self {
      inner: context,
      _marker: PhantomData,
    };
    Rc::new(context)
  }

  pub(crate) fn inner(&self) -> *mut qjs::JSContext {
    self.inner
  }

  /// is_module: module or global
  pub fn eval(
    self: Rc<Self>,
    code: &str,
    name: &str,
    is_module: bool,
    compile_only: bool,
  ) -> Result<JsValue, JsError> {
    let eval_flags = match (is_module, compile_only) {
      (true, true) => qjs::JS_EVAL_TYPE_MODULE | qjs::JS_EVAL_FLAG_COMPILE_ONLY,
      (true, false) => qjs::JS_EVAL_TYPE_MODULE,
      (false, true) => {
        qjs::JS_EVAL_TYPE_GLOBAL | qjs::JS_EVAL_FLAG_COMPILE_ONLY
      }
      (false, false) => qjs::JS_EVAL_TYPE_GLOBAL,
    } as _;
    let code_cstring = CString::new(code).unwrap();
    let input = code_cstring.as_ptr();
    let input_len = code.len() as _;
    let name_cstring = CString::new(name).unwrap();
    let filename = name_cstring.as_ptr();

    let value = unsafe {
      qjs::JS_Eval(self.inner, input, input_len, filename, eval_flags)
    };
    let value = JsValue::from_qjs(Rc::clone(&self), value);

    if value.is_exception() {
      return Err(self.into());
    }
    Ok(value)
  }

  pub fn eval_function(self: Rc<Self>, func_obj: &JsValue) -> JsValue {
    let value = unsafe { qjs::JS_EvalFunction(self.inner, func_obj.inner()) };
    JsValue::from_qjs(self, value)
  }
}
