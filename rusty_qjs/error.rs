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
      let name = value.get_property_str("name").into();
      let message = value.get_property_str("message").into();
      let stack = value.get_property_str("stack").into();

      value.free();

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
