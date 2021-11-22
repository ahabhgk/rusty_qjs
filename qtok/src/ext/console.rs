use std::io::Write;

use rusty_qjs::{CallContext, JSContext, JSValue};
use rusty_qjs_derive::js_function;

use crate::error::JSException;

#[js_function]
fn print(mut call_ctx: CallContext) -> JSValue {
  let mut stdout = std::io::stdout();
  for i in 0..call_ctx.argc {
    if i != 0 {
      stdout.write_all(b" ").unwrap();
    }
    let val = call_ctx.get(i).unwrap();
    stdout
      .write_all(val.to_string(call_ctx.js_context).as_bytes())
      .unwrap();
  }
  stdout.write_all(b"\n").unwrap();
  JSValue::new_undefined()
}

pub fn add_console(ctx: &mut JSContext) -> Result<(), JSException> {
  let global_obj = ctx.get_global_object().to_local(ctx);
  let console = JSValue::new_object(ctx).to_local(ctx);
  let func = JSValue::new_function(ctx, print, "log", 1).to_local(ctx);

  console.set_property_str("log", func)?;
  global_obj.set_property_str("console", console)?;

  Ok(())
}
