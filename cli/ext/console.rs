use std::io::Write;

use rusty_qjs::{context::JsContext, value::JsValue};

use crate::error::AnyError;

fn js_print(
  ctx: *mut libquickjs_sys::JSContext,
  _this_val: libquickjs_sys::JSValue,
  argc: i32,
  argv: *mut libquickjs_sys::JSValue,
) -> libquickjs_sys::JSValue {
  let mut ctx = JsContext::from_raw(ctx);
  let mut stdout = std::io::stdout();
  for i in 0..argc {
    if i != 0 {
      stdout.write(b" ").unwrap();
    }
    let raw_value = unsafe { *argv.offset(i as isize) };
    let val = JsValue::from_raw(unsafe { ctx.0.as_mut() }, raw_value);
    stdout.write(String::from(val).as_bytes()).unwrap();
  }
  stdout.write(b"\n").unwrap();
  JsValue::new_undefined(&mut ctx).raw_value
}

pub fn add_console(ctx: &mut JsContext) -> Result<(), AnyError> {
  let mut global_obj = ctx.get_global_object();
  let mut console = JsValue::new_object(ctx);
  let func = JsValue::new_function(ctx, js_print, "log", 1);
  console.set_property_str("log", func)?;
  global_obj.set_property_str("console", console)?;
  global_obj.free();
  Ok(())
}
