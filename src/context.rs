use std::{
  ffi::CString,
  marker::PhantomData,
  ops::{Deref, DerefMut},
  ptr::NonNull,
};

use crate::{
  runtime::{JSRuntime, OwnedJSRuntime},
  support::Opaque,
  sys, Local,
};

extern "C" {
  pub fn JS_NewContext(rt: *mut JSRuntime) -> *mut JSContext;
}

// #[derive(Debug)]
// pub struct JsContext {
//   pub raw_context: *mut sys::JSContext,
// }

// // impl QuickjsRc for JsContext {
// //   fn free(&mut self) {
// //     unsafe { sys::JS_FreeContext(self.raw_context) };
// //   }

// //   fn dup(&self) -> Self {
// //     let raw_context = unsafe { sys::JS_DupContext(self.raw_context) };
// //     Self { raw_context }
// //   }
// // }

// impl JsContext {
//   pub fn free(&mut self) {
//     unsafe { sys::JS_FreeContext(self.raw_context) };
//   }

//   pub fn dup(&self) -> Self {
//     let raw_context = unsafe { sys::JS_DupContext(self.raw_context) };
//     Self { raw_context }
//   }

//   pub fn new(runtime: &JsRuntime) -> Self {
//     let ctx = unsafe { sys::JS_NewContext(runtime.raw_runtime) };
//     Self::from_raw(ctx)
//   }

//   pub fn from_raw(raw_context: *mut sys::JSContext) -> Self {
//     Self { raw_context }
//   }

//   pub fn eval_module(&self, code: &str, name: &str) -> Local<JsValue> {
//     self.eval(code, name, true, false)
//   }

//   pub fn compile_module(&self, code: &str, name: &str) -> Local<JsValue> {
//     self.eval(code, name, true, true)
//   }

//   pub fn eval_script(&self, code: &str, name: &str) -> Local<JsValue> {
//     self.eval(code, name, false, false)
//   }

//   pub fn compile_script(&self, code: &str, name: &str) -> Local<JsValue> {
//     self.eval(code, name, false, true)
//   }

//   fn eval(
//     &self,
//     code: &str,
//     name: &str,
//     is_module: bool,
//     compile_only: bool,
//   ) -> Local<JsValue> {
//     let eval_flags = match (is_module, compile_only) {
//       (true, true) => sys::JS_EVAL_TYPE_MODULE | sys::JS_EVAL_FLAG_COMPILE_ONLY,
//       (true, false) => sys::JS_EVAL_TYPE_MODULE,
//       (false, true) => {
//         sys::JS_EVAL_TYPE_GLOBAL | sys::JS_EVAL_FLAG_COMPILE_ONLY
//       }
//       (false, false) => sys::JS_EVAL_TYPE_GLOBAL,
//     } as _;
//     let code_cstring = CString::new(code).unwrap();
//     let input = code_cstring.as_ptr();
//     let input_len = code.len() as _;
//     let name_cstring = CString::new(name).unwrap();
//     let filename = name_cstring.as_ptr();

//     let value = unsafe {
//       sys::JS_Eval(self.raw_context, input, input_len, filename, eval_flags)
//     };
//     Local::from(JsValue::from_raw(self.raw_context, value))
//   }

//   pub fn eval_function(&self, func_obj: &JsValue) -> Local<JsValue> {
//     let raw_context = self.raw_context;
//     let value =
//       unsafe { sys::JS_EvalFunction(raw_context, func_obj.raw_value) };
//     Local::from(JsValue::from_raw(raw_context, value))
//   }

//   pub fn get_exception(&self) -> Local<JsValue> {
//     let raw_context = self.raw_context;
//     let exception = unsafe { sys::JS_GetException(raw_context) };
//     Local::from(JsValue::from_raw(raw_context, exception))
//   }

//   pub fn get_global_object(&self) -> Local<JsValue> {
//     let raw_context = self.raw_context;
//     let global_object = unsafe { sys::JS_GetGlobalObject(raw_context) };
//     Local::from(JsValue::from_raw(raw_context, global_object))
//   }
// }

////////////////////

#[repr(C)]
#[derive(Debug, Copy, Clone)] // Clone?
pub struct JSContext(Opaque);

impl JSContext {
  pub fn new<'rt>(rt: &'rt mut OwnedJSRuntime) -> OwnedJSContext {
    let rt = rt.deref_mut();
    let ctx = unsafe { JS_NewContext(rt) };
    let ctx = NonNull::new(ctx).unwrap();
    OwnedJSContext(ctx, PhantomData)
  }
}

pub struct OwnedJSContext<'rt>(NonNull<JSContext>, PhantomData<&'rt ()>);

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
