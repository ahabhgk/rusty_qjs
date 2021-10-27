use std::{error::Error, fmt::Display};

use crate::value::JsValue;

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
