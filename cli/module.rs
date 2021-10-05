use crate::error::AnyError;
use libquickjs_sys as qjs;
use std::{cell::RefCell, rc::Rc};

pub fn js_module_set_import_meta(
    ctx: Rc<RefCell<qjs::JSContext>>,
    module: &qjs::JSValue,
    use_realpath: bool,
    is_main: bool,
) -> Result<(), AnyError> {
    todo!()
}
