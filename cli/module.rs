use crate::error::AnyError;
use libquickjs_sys as qjs;

pub fn js_module_set_import_meta(
    ctx: *mut qjs::JSContext,
    module: &qjs::JSValue,
    use_realpath: bool,
    is_main: bool,
) -> Result<(), AnyError> {
    // TODO
    Ok(())
}
