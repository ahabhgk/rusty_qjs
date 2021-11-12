use crate::{context::JsContext, error::Error, handle::Local, value::JsValue};

pub struct CallContext<'ctx> {
  pub js_context: &'ctx mut JsContext,
  raw_this: libquickjs_sys::JSValue,
  pub argc: i32,
  argv: *mut libquickjs_sys::JSValue,
}

impl<'ctx> CallContext<'ctx> {
  pub fn new(
    js_context: &'ctx mut JsContext,
    raw_this: libquickjs_sys::JSValue,
    argc: i32,
    argv: *mut libquickjs_sys::JSValue,
  ) -> Self {
    Self {
      js_context,
      raw_this,
      argc,
      argv,
    }
  }

  pub fn get(&self, index: i32) -> Result<Local<JsValue>, Error> {
    if index >= self.argc {
      Err(Error::ArgumentsIndexOutOfRange)
    } else {
      Ok(Local::new(JsValue::from_raw(
        self.js_context.raw_context,
        unsafe { *self.argv.offset(index as isize) },
      )))
    }
  }

  pub fn this(&self) -> Local<JsValue> {
    Local::new(JsValue::from_raw(
      self.js_context.raw_context,
      self.raw_this,
    ))
  }
}
