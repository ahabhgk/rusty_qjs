use crate::error::AnyError;
use rusty_qjs::{context::JsContext, value::JsValue};

pub fn js_module_set_import_meta(
  ctx: &JsContext,
  module: &JsValue,
  use_realpath: bool,
  is_main: bool,
) -> Result<(), AnyError> {
  // TODO
  Ok(())
}
