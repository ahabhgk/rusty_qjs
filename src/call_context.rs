use crate::{error::ArgumentsIndexOutOfRange, JSContext, JSValue};

// TODO: delete this, use repr(C) type and fn_mapping
/// function call context.
pub struct CallContext<'ctx> {
  /// JSContext of the function.
  pub js_context: &'ctx mut JSContext,
  raw_this: JSValue,
  /// Number of the arguments.
  pub argc: i32,
  argv: *mut JSValue,
}

impl<'ctx> CallContext<'ctx> {
  /// Create a function call context.
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

  /// Get the argument of the function call context by index.
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

  /// Get the this of the function call context.
  pub fn this(&mut self) -> JSValue {
    self.raw_this
  }
}
