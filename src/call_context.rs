use crate::{error::ArgumentsIndexOutOfRange, JSContext, JSValue};

pub struct CallContext<'ctx> {
  pub js_context: &'ctx mut JSContext,
  raw_this: JSValue,
  pub argc: i32,
  argv: *mut JSValue,
}

impl<'ctx> CallContext<'ctx> {
  pub fn new(
    js_context: &'ctx mut JSContext,
    raw_this: JSValue,
    argc: i32,
    argv: *mut JSValue,
  ) -> Self {
    Self {
      js_context,
      raw_this,
      argc,
      argv,
    }
  }

  pub fn get(
    &mut self,
    index: i32,
  ) -> Result<JSValue, ArgumentsIndexOutOfRange> {
    if index >= self.argc {
      Err(ArgumentsIndexOutOfRange)
    } else {
      let arg = unsafe { *self.argv.offset(index as isize) };
      Ok(arg)
    }
  }

  pub fn this(&mut self) -> JSValue {
    self.raw_this
  }
}
