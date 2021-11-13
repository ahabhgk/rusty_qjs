use std::io::Write;

use rusty_qjs::{
  call_context::CallContext, context::JsContext, handle::Local, value::JsValue,
};
use rusty_qjs_derive::js_function;

use crate::error::AnyError;

#[js_function]
fn print(call_ctx: CallContext) -> Local<JsValue> {
  let mut stdout = std::io::stdout();
  for i in 0..call_ctx.argc {
    if i != 0 {
      stdout.write(b" ").unwrap();
    }
    let val = call_ctx.get(i).unwrap();
    stdout.write(String::from(val).as_bytes()).unwrap();
  }
  stdout.write(b"\n").unwrap();
  JsValue::new_undefined(&call_ctx.js_context)
}

pub fn add_console(ctx: &JsContext) -> Result<(), AnyError> {
  let global_obj = ctx.get_global_object();
  let console = JsValue::new_object(ctx);
  let func = JsValue::new_function(ctx, print, "log", 1);

  console.set_property_str("log", func)?;
  global_obj.set_property_str("console", console)?;

  Ok(())
}
