use std::ffi::CStr;

pub type Opaque = [u8; 0];

pub fn jsbool_to_bool(js_bool: libc::c_int) -> bool {
  js_bool != 0
}

pub fn cstr_to_string(cstr: &CStr) -> String {
  String::from_utf8_lossy(cstr.to_bytes()).into_owned()
}
