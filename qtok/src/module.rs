use crate::error::AnyError;
use rusty_qjs::{JsContext, JsValue};

pub fn js_module_set_import_meta(
  _ctx: &JsContext,
  _module: &JsValue,
  _use_realpath: bool,
  _is_main: bool,
) -> Result<(), AnyError> {
  // TODO
  Ok(())
}
