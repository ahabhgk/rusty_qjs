use std::{error::Error, fmt::Display};

use rusty_qjs::{error::JSContextException, QuickjsRc};

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
    let (name, message, stack) = if value.is_error(ctx) {
      let mut v = value.get_property_str(ctx, "name");
      let name = v.to_string(ctx);
      v.free(ctx);

      let mut v = value.get_property_str(ctx, "message");
      let message = v.to_string(ctx);
      v.free(ctx);

      let mut v = value.get_property_str(ctx, "stack");
      let stack = v.to_string(ctx);
      v.free(ctx);

      (name, message, stack)
    } else {
      let message = value.to_string(ctx);
      ("Exception".to_owned(), message, "".to_owned())
    };

    Self {
      name,
      stack,
      message,
    }
  }
}
