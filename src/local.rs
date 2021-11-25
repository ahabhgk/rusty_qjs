use std::{
  marker::PhantomData,
  mem,
  ops::{Deref, DerefMut},
};

use crate::{error::JSContextException, JSContext, JSValue, QuickjsRc};

/// Same as JSValue but gets freed when it drops and duplicated when it clones.
pub struct Local<'ctx> {
  /// JSValue of the Local.
  pub value: JSValue,
  context: *mut JSContext,
  _marker: PhantomData<&'ctx mut JSContext>,
}

impl Drop for Local<'_> {
  fn drop(&mut self) {
    let ctx = self.get_context_mut();
    self.value.free(ctx)
  }
}

impl JSValue {
  /// Convert the JSValue to Local
  pub fn to_local<'ctx>(self, ctx: &mut JSContext) -> Local<'ctx> {
    Local::new(ctx, self)
  }
}

impl Deref for Local<'_> {
  type Target = JSValue;

  fn deref(&self) -> &Self::Target {
    &self.value
  }
}

impl DerefMut for Local<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.value
  }
}

impl From<Local<'_>> for String {
  fn from(lc: Local) -> Self {
    let ctx = unsafe { lc.context.as_mut() }.unwrap();
    lc.value.to_rust_string(ctx).unwrap()
  }
}

impl<'ctx> From<Local<'ctx>> for JSContextException<'ctx> {
  fn from(lc: Local<'ctx>) -> Self {
    let context = lc.get_context_mut();
    let value = lc.value;
    mem::forget(lc);
    Self { value, context }
  }
}

impl<'ctx> Local<'ctx> {
  /// Create a Local by JSValue and its JSContext.
  pub fn new(ctx: &mut JSContext, value: JSValue) -> Self {
    Self {
      value,
      context: ctx,
      _marker: PhantomData,
    }
  }

  /// Clone the Local, and increment the reference count of the JValue in it.
  pub fn dup(&mut self) -> Self {
    let ctx = self.get_context_mut();
    let value = self.value.dup(ctx);
    Self::new(ctx, value)
  }

  /// Get the &mut JSContext of the Local.
  pub fn get_context_mut(&self) -> &'ctx mut JSContext {
    unsafe { self.context.as_mut() }.unwrap()
  }

  /// Returns true if the Local is an error.
  pub fn is_error(&self) -> bool {
    let ctx = self.get_context_mut();
    self.value.is_error(ctx)
  }

  /// Same as JSValue::get_property_str, but for Local type.
  pub fn get_property_str(&self, prop: &str) -> Self {
    let ctx = self.get_context_mut();
    self.value.get_property_str(ctx, prop).to_local(ctx)
  }

  /// Same as JSValue::get_property_str, but for Local type.
  pub fn set_property_str(
    &self,
    prop: &str,
    value: Self,
  ) -> Result<bool, JSContextException> {
    let ctx = self.get_context_mut();
    let ret = self.value.set_property_str(ctx, prop, value.value);
    mem::forget(value);
    ret
  }
}

#[cfg(test)]
mod tests {
  use std::io::Write;

  use crate::JSRuntime;

  use super::*;

  // TODO: test though assert JSValue reference count number
  #[test]
  fn global_console_log() {
    let rt = &mut JSRuntime::new();
    let ctx = &mut JSContext::new(rt);
    // setup console.log
    {
      fn js_print(
        ctx: &mut JSContext,
        _this: JSValue,
        argv: &[JSValue],
      ) -> JSValue {
        let output = argv
          .iter()
          .map(|value| value.to_rust_string(ctx).unwrap())
          .collect::<Vec<String>>()
          .join(" ");
        let mut stdout = std::io::stdout();
        stdout.write_all(output.as_bytes()).unwrap();
        stdout.write_all(b"\n").unwrap();
        JSValue::new_undefined()
      }

      let global = ctx.get_global_object().to_local(ctx);

      let console = JSValue::new_object(ctx).to_local(ctx);
      let log = JSValue::new_function(ctx, js_print, "log", 1).to_local(ctx);

      console.set_property_str("log", log).unwrap();
      global.set_property_str("console", console).unwrap();
    }

    ctx.eval_script("console.log('hello world')", "<test>");
  }
}
