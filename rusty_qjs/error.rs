use super::{context::JsContext, value::JsValue};
use std::{error::Error, fmt::Display, ptr::NonNull};

#[derive(Debug, Clone)]
pub struct JsError {
  stack: String,
  message: String,
  name: String,
}

impl From<JsValue> for JsError {
  fn from(mut value: JsValue) -> Self {
    let (name, message, stack) = if value.is_error() {
      let name = match value.get_property("name") {
        Some(v) => v.into(),
        None => "Error".to_owned(),
      };

      let message = match value.get_property("message") {
        Some(v) => v.into(),
        None => "".to_owned(),
      };

      let stack = match value.get_property("stack") {
        Some(v) => v.into(),
        None => "".to_owned(),
      };

      (name, message, stack)
    } else {
      let message = String::from(value);
      ("Exception".to_owned(), message, "".to_owned())
    };

    Self {
      name,
      stack,
      message,
    }
  }
}

impl JsError {
  // FIXME: move dump to cli
  pub fn dump_from_context(ctx: &mut JsContext) -> Self {
    JsError::dump_from_raw_context(unsafe { ctx.0.as_mut() })
  }

  pub fn dump_from_raw_context(ctx: *mut libquickjs_sys::JSContext) -> Self {
    let exception = unsafe { libquickjs_sys::JS_GetException(ctx) };
    let ctx = NonNull::new(ctx).unwrap();
    let exception = JsValue {
      ctx,
      val: exception,
    };
    exception.into()
  }
}

impl Error for JsError {}

impl Display for JsError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}\n{}", self.name, self.message, self.stack)
  }
}

// impl TryFrom<qjs::JSValue> for JsError {
//     type Error = AnyError;

//     fn try_from(value: qjs::JSValue) -> Result<Self, Self::Error> {
//         let is_exception = unsafe { qjs::JS_IsException(value) };
//         if is_exception {
//             return Err(AnyError::msg("is not an error"));
//         }
//         Ok(Self::from(value))
//     }
// }
