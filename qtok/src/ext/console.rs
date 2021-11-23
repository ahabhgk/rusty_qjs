use std::io::Write;

use rusty_qjs::{JSContext, JSValue};

use crate::error::JSException;

fn js_print(ctx: &mut JSContext, _this: JSValue, argv: &[JSValue]) -> JSValue {
  let output = argv
    .iter()
    .map(|value| value.to_string(ctx))
    .collect::<Vec<String>>()
    .join(" ");
  let mut stdout = std::io::stdout();
  stdout.write_all(output.as_bytes()).unwrap();
  stdout.write_all(b"\n").unwrap();
  JSValue::new_undefined()
}

pub fn add_console(ctx: &mut JSContext) -> Result<(), JSException> {
  let global_obj = ctx.get_global_object().to_local(ctx);
  let console = JSValue::new_object(ctx).to_local(ctx);
  let func = JSValue::new_function(ctx, js_print, "log", 1).to_local(ctx);

  console.set_property_str("log", func)?;
  global_obj.set_property_str("console", console)?;

  Ok(())
}
