use std::{error::Error, fmt::Display};

use crate::{JSContext, JSValue};

#[derive(Debug)]
pub struct JSContextException<'ctx> {
  pub value: JSValue,
  pub context: &'ctx mut JSContext,
}

impl Error for JSContextException<'_> {}

impl Display for JSContextException<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "JSContextException: {:?}", self.value)
  }
}

#[derive(Debug)]
pub struct ArgumentsIndexOutOfRange;

impl Error for ArgumentsIndexOutOfRange {}

impl Display for ArgumentsIndexOutOfRange {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Arguments index out of range")
  }
}
