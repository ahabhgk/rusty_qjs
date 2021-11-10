use std::io::Write;

use rusty_qjs::{
  call_context::CallContext, context::JsContext, value::JsValue,
};

use crate::error::AnyError;

fn js_print(
  ctx: *mut libquickjs_sys::JSContext,
  this_val: libquickjs_sys::JSValue,
  argc: i32,
  argv: *mut libquickjs_sys::JSValue,
) -> libquickjs_sys::JSValue {
  let mut ctx = JsContext::from_raw(ctx);
  let mut call_ctx = CallContext::new(&mut ctx, this_val, argc, argv);

  let mut stdout = std::io::stdout();
  for i in 0..argc {
    if i != 0 {
      stdout.write(b" ").unwrap();
    }
    let val = call_ctx.get(i).unwrap();
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
