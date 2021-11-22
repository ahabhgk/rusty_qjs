//! Errors for working with rusty_qjs.

use std::{error::Error, fmt::Display};

use crate::{JSContext, JSValue};

/// Exception from JSContext, Returns this Error when quickjs side
/// function runs into an error.
#[derive(Debug)]
pub struct JSContextException<'ctx> {
  /// The exception JSValue.
  pub value: JSValue,
  /// The JSContext of the exception JSValue.
  pub context: &'ctx mut JSContext,
}

impl Error for JSContextException<'_> {}

impl Display for JSContextException<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "JSContextException: {:?}", self.value)
  }
}

/// Returns this Error when CallContext::get argument index out of range.
#[derive(Debug)]
pub struct ArgumentsIndexOutOfRange;

impl Error for ArgumentsIndexOutOfRange {}

impl Display for ArgumentsIndexOutOfRange {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Arguments index out of range")
  }
}
