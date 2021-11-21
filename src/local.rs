use std::{marker::PhantomData, mem};

use crate::{
  error::JSContextException, value::JSFunction, JSContext, JSValue, QuickjsRc,
};

pub struct Local<'ctx> {
  value: JSValue,
  context: *mut JSContext,
  _marker: PhantomData<&'ctx mut JSContext>,
}

impl Drop for Local<'_> {
  fn drop(&mut self) {
    let ctx = unsafe { self.context.as_mut() }.unwrap();
    self.value.free(ctx)
  }
}

impl Clone for Local<'_> {
  fn clone(&self) -> Self {
    let ctx = unsafe { self.context.as_mut() }.unwrap();
    let value = self.value.dup(ctx);
    Self::new(ctx, value)
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

  pub fn new_undefined(ctx: &mut JSContext) -> Self {
    let value = JSValue::new_undefined();
    Self::new(ctx, value)
  }

  pub fn new_object(ctx: &mut JSContext) -> Self {
    let value = JSValue::new_object(ctx);
    Self::new(ctx, value)
  }

  pub fn new_function(
    ctx: &mut JSContext,
    func: JSFunction,
    name: &str,
    len: i32,
  ) -> Self {
    let value = JSValue::new_function(ctx, func, name, len);
    Self::new(ctx, value)
  }

  pub fn is_error(&self) -> bool {
    let ctx = unsafe { self.context.as_mut() }.unwrap();
    self.value.is_error(ctx)
  }

  pub fn is_exception(&self) -> bool {
    self.value.is_exception()
  }

  pub fn is_undefined(&self) -> bool {
    self.value.is_undefined()
  }

  pub fn get_property_str(&self, prop: &str) -> Self {
    let ctx = unsafe { self.context.as_mut() }.unwrap();
    let value = self.value.get_property_str(ctx, prop);
    Self::new(ctx, value)
  }

  pub fn set_property_str(
    &self,
    prop: &str,
    value: Self,
  ) -> Result<bool, JSContextException> {
    let ctx = unsafe { self.context.as_mut() }.unwrap();
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
      extern "C" fn print(
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

      let global = ctx.get_global_object();
      let global = Local::new(ctx, global);

      let console = Local::new_object(ctx);
      let func = Local::new_function(ctx, print, "log", 1);

      console.set_property_str("log", func).unwrap();
      global.set_property_str("console", console).unwrap();
    }

    ctx.eval_script("console.log(\"hello world\")", "<test>");
  }
}
