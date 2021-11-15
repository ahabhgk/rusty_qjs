pub mod js_value;

// use std::{
//   ffi::{CStr, CString},
//   fmt, ptr,
// };

// use crate::{sys, Error, JsContext, Local, QuickjsRc};

// type JsFunction = extern "C" fn(
//   *mut sys::JSContext,
//   sys::JSValue,
//   i32,
//   *mut sys::JSValue,
// ) -> sys::JSValue;

// pub struct JsValue {
//   pub raw_value: sys::JSValue,
//   pub raw_context: *mut sys::JSContext,
// }

// // TODO: move sys into this crate, impl Debug for JSValue
// impl fmt::Debug for JsValue {
//   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//     let tag = match self.raw_value.tag as i32 {
//       sys::JS_TAG_BIG_DECIMAL => "BigDecimal",
//       sys::JS_TAG_BIG_INT => "BigInt",
//       sys::JS_TAG_BIG_FLOAT => "BigFloat",
//       sys::JS_TAG_SYMBOL => "Symbol",
//       sys::JS_TAG_STRING => "String",
//       sys::JS_TAG_MODULE => "Module (internal)",
//       sys::JS_TAG_FUNCTION_BYTECODE => "FunctionBytecode (internal)",
//       sys::JS_TAG_OBJECT => "Object",
//       sys::JS_TAG_INT => "Int",
//       sys::JS_TAG_BOOL => "Bool",
//       sys::JS_TAG_NULL => "Null",
//       sys::JS_TAG_UNDEFINED => "Undefined",
//       sys::JS_TAG_UNINITIALIZED => "Uninitialized",
//       sys::JS_TAG_CATCH_OFFSET => "CatchOffset",
//       sys::JS_TAG_EXCEPTION => "Exception",
//       sys::JS_TAG_FLOAT64 => "Float64",
//       _ => "Unknown (unexpected)",
//     };
//     write!(
//       f,
//       r#"JsValue {{
//   context: {:p},
//   inner: {{
//     u: {{
//       int32: {:?}, float64: {:?},
//       ptr: {:p}
//     }},
//     tag: {:?},
//   }},
// }}"#,
//       self.raw_context,
//       unsafe { self.raw_value.u.int32 },
//       unsafe { self.raw_value.u.float64 },
//       unsafe { self.raw_value.u.ptr },
//       tag,
//     )
//   }
// }

// impl QuickjsRc for JsValue {
//   fn free(&mut self) {
//     // JS_TAG_MODULE never freed, see quickjs.c#L5518
//     if self.raw_value.tag == sys::JS_TAG_MODULE.into() {
//       return;
//     }
//     unsafe { sys::JS_FreeValue(self.raw_context, self.raw_value) };
//   }

//   fn dup(&self) -> Self {
//     let raw_value =
//       unsafe { sys::JS_DupValue(self.raw_context, self.raw_value) };
//     Self::from_raw(self.raw_context, raw_value)
//   }
// }

// impl From<Local<JsValue>> for String {
//   fn from(value: Local<JsValue>) -> Self {
//     let value = value.to_qjsrc();
//     let ptr = unsafe {
//       sys::JS_ToCStringLen2(
//         value.raw_context,
//         ptr::null_mut(),
//         value.raw_value,
//         0,
//       ) as *mut _
//     };
//     let cstr = unsafe { CStr::from_ptr(ptr) };
//     unsafe { sys::JS_FreeCString(value.raw_context, ptr) };
//     let s = cstr.to_str().unwrap();
//     s.to_owned()
//   }
// }

// impl From<Local<JsValue>> for Error {
//   fn from(value: Local<JsValue>) -> Self {
//     let (name, message, stack) = if value.is_error() {
//       let name = value.get_property_str("name").into();
//       let message = value.get_property_str("message").into();
//       let stack = value.get_property_str("stack").into();

//       (name, message, stack)
//     } else {
//       let message = String::from(value);
//       ("Exception".to_owned(), message, "".to_owned())
//     };

//     Self::JSContextError {
//       name,
//       stack,
//       message,
//     }
//   }
// }

// impl JsValue {
//   pub fn from_raw(
//     raw_context: *mut sys::JSContext,
//     raw_value: sys::JSValue,
//   ) -> Self {
//     Self {
//       raw_context,
//       raw_value,
//     }
//   }

//   pub fn new_object(ctx: &JsContext) -> Local<Self> {
//     let raw_context = ctx.raw_context;
//     let obj = unsafe { sys::JS_NewObject(raw_context) };
//     Local::from(Self::from_raw(raw_context, obj))
//   }

//   pub fn new_function(
//     ctx: &JsContext,
//     func: JsFunction,
//     name: &str,
//     len: i32,
//   ) -> Local<Self> {
//     let raw_context = ctx.raw_context;
//     let name_cstring = CString::new(name).unwrap();
//     let val = unsafe {
//       sys::JS_NewCFunction(
//         raw_context,
//         std::mem::transmute(func as *mut ()),
//         name_cstring.as_ptr(),
//         len,
//       )
//     };
//     Local::from(Self::from_raw(raw_context, val))
//   }

//   pub fn new_undefined(ctx: &JsContext) -> Local<Self> {
//     let raw_context = ctx.raw_context;
//     let val = sys::JSValue {
//       u: sys::JSValueUnion { int32: 0 },
//       tag: sys::JS_TAG_UNDEFINED.into(),
//     };
//     Local::from(Self::from_raw(raw_context, val))
//   }

//   pub fn get_property_str(&self, prop: &str) -> Local<Self> {
//     let prop_cstring = CString::new(prop).unwrap();
//     let raw_value = unsafe {
//       sys::JS_GetPropertyStr(
//         self.raw_context,
//         self.raw_value,
//         prop_cstring.as_ptr(),
//       )
//     };
//     Local::from(Self::from_raw(self.raw_context, raw_value))
//   }

//   pub fn set_property_str(
//     &self,
//     prop: &str,
//     value: Local<JsValue>,
//   ) -> Result<bool, Error> {
//     let value = value.to_qjsrc();
//     let prop_cstring = CString::new(prop).unwrap();
//     let result = unsafe {
//       sys::JS_SetPropertyStr(
//         self.raw_context,
//         self.raw_value,
//         prop_cstring.as_ptr(),
//         value.raw_value,
//       )
//     };
//     match result {
//       -1 => Err(JsContext::from_raw(self.raw_context).get_exception().into()),
//       0 => Ok(false),
//       1 => Ok(true),
//       _ => panic!("JS_SetPropertyStr return unexpected"),
//     }
//   }

//   pub fn is_error(&self) -> bool {
//     unsafe { sys::JS_IsError(self.raw_context, self.raw_value) == 1 }
//   }

//   pub fn is_exception(&self) -> bool {
//     unsafe { sys::JS_IsException(self.raw_value) }
//   }

//   pub fn is_undefined(&self) -> bool {
//     unsafe { sys::JS_IsUndefined(self.raw_value) }
//   }
// }
