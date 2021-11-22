use std::{
  marker::PhantomData,
  mem,
  ops::{Deref, DerefMut},
};

use crate::{error::JSContextException, JSContext, JSValue, QuickjsRc};

pub struct Local<'ctx> {
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

impl Clone for Local<'_> {
  fn clone(&self) -> Self {
    let ctx = self.get_context_mut();
    let value = self.value.dup(ctx);
    Self::new(ctx, value)
  }
}

impl JSValue {
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
    lc.value.to_string(ctx)
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
  pub fn new(ctx: &mut JSContext, value: JSValue) -> Self {
    Self {
      value,
      context: ctx,
      _marker: PhantomData,
    }
  }

  pub fn get_context_mut(&self) -> &'ctx mut JSContext {
    unsafe { self.context.as_mut() }.unwrap()
  }

  pub fn is_error(&self) -> bool {
    let ctx = self.get_context_mut();
    self.value.is_error(ctx)
  }

  pub fn get_property_str(&self, prop: &str) -> Self {
    let ctx = self.get_context_mut();
    self.value.get_property_str(ctx, prop).to_local(ctx)
  }

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
  fn global_console() {
    let rt = &mut JSRuntime::new();
    let ctx = &mut JSContext::new(rt);
    // setup console.log
    {
      extern "C" fn js_print(
        ctx: *mut crate::JSContext,
        this_val: crate::JSValue,
        argc: i32,
        argv: *mut crate::JSValue,
      ) -> crate::JSValue {
        let mut ctx = unsafe { ctx.as_mut() }.unwrap();
        let mut call_ctx =
          crate::CallContext::new(&mut ctx, this_val, argc, argv);

        // real function
        let mut stdout = std::io::stdout();
        for i in 0..call_ctx.argc {
          if i != 0 {
            stdout.write(b" ").unwrap();
          }
          let val = call_ctx.get(i).unwrap();
          stdout
            .write(val.to_string(call_ctx.js_context).as_bytes())
            .unwrap();
        }
        stdout.write(b"\n").unwrap();
        JSValue::new_undefined()
      }

      let global = ctx.get_global_object().to_local(ctx);

      let console = JSValue::new_object(ctx).to_local(ctx);
      let log = JSValue::new_function(ctx, js_print, "log", 1).to_local(ctx);

      console.set_property_str("log", log).unwrap();
      global.set_property_str("console", console).unwrap();
    }

    ctx.eval_script("console.log(\"hello world\")", "<test>");
  }
}
