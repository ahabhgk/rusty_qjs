use crate::error::AnyError;
use rusty_qjs::{context::JsContext, value::JsValue};
use std::rc::Rc;

pub fn js_module_set_import_meta(
  ctx: Rc<JsContext>,
  module: &JsValue,
  use_realpath: bool,
  is_main: bool,
) -> Result<(), AnyError> {
  // TODO
  Ok(())
}
