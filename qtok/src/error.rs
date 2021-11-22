use std::{error::Error, fmt::Display};

use rusty_qjs::{error::JSContextException, Local};

pub type AnyError = anyhow::Error;

#[derive(Debug, Clone)]
pub struct JSException {
  name: String,
  message: String,
  stack: String,
}

impl Error for JSException {}

impl Display for JSException {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}\n{}", self.name, self.message, self.stack)
  }
}

impl From<JSContextException<'_>> for JSException {
  fn from(error: JSContextException) -> Self {
    let JSContextException {
      value,
      context: ctx,
    } = error;
    let value = Local::new(ctx, value);
    let (name, message, stack) = if value.is_error() {
      let name = value.get_property_str("name").into();
      let message = value.get_property_str("message").into();
      let stack = value.get_property_str("stack").into();

      (name, message, stack)
    } else {
      let message = value.into();
      ("Exception".to_owned(), message, "".to_owned())
    };

    Self {
      name,
      stack,
      message,
    }
  }
}
