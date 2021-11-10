use std::{ffi::c_void, fs, path::Path, task::Poll};

use futures::future::poll_fn;
use rusty_qjs::{
  context::JsContext,
  runtime::JsRuntime,
  value::{error::JsError, JsValue},
};

use crate::{error::AnyError, ext, module::js_module_set_import_meta};

extern "C" fn host_promise_rejection_tracker(
  ctx: *mut libquickjs_sys::JSContext,
  _promise: libquickjs_sys::JSValue,
  reason: libquickjs_sys::JSValue,
  is_handled: ::std::os::raw::c_int,
  opaque: *mut ::std::os::raw::c_void,
) {
  if is_handled == 0 {
    let qtok = unsafe { &mut *(opaque as *mut Qtok) };
    unsafe { libquickjs_sys::JS_DupValue(ctx, reason) };
    let reason = JsValue::from_raw(ctx, reason);
    qtok.pending_promise_exceptions.push(JsError::from(reason))
  }
}

pub struct Qtok {
  js_context: JsContext,
  js_runtime: JsRuntime,
  pending_promise_exceptions: Vec<JsError>,
  // pending_ops:
}

impl Drop for Qtok {
  fn drop(&mut self) {
    self.js_context.free();
    self.js_runtime.free();
  }
}

impl Qtok {
  pub fn new() -> Self {
    let mut js_runtime = JsRuntime::default();
    let js_context = JsContext::new(&mut js_runtime);
    let mut qtok = Self {
      js_context,
      js_runtime,
      pending_promise_exceptions: Vec::new(),
    };
    // JS_SetMaxStackSize
    // JS_SetModuleLoaderFunc
    // JS_SetHostPromiseRejectionTracker
    let opaque = { &mut qtok as *mut _ as *mut c_void };
    unsafe {
      qtok.js_runtime.set_host_promise_rejection_tracker(
        Some(host_promise_rejection_tracker),
        opaque,
      )
    };
    // js_init_module_uv core, timers, error, fs, process...
    // tjs__bootstrap_globals fetch, url, performance, console, wasm...
    // tjs__add_builtins path, uuid, hashlib...
    ext::console::add_console(&mut qtok.js_context).unwrap();

    qtok
  }

  pub fn eval_module(
    &mut self,
    path: &Path,
    is_main: bool,
  ) -> Result<(), AnyError> {
    let _ = self.eval_file(path, is_main)?;
    Ok(())
  }

  fn eval_file(
    &mut self,
    path: &Path,
    is_main: bool,
  ) -> Result<JsValue, AnyError> {
    let code = fs::read_to_string(path)?;
    let code = &code[..];
    let name = path.to_str().unwrap();
    let ctx = &mut self.js_context;

    let mut ret = ctx.compile_module(code, name);
    if ret.is_exception() {
      return Err(self.dump_error().into());
    }

    js_module_set_import_meta(ctx, &ret, true, is_main)?;

    // TODO: eval module, continue abstract eval?
    ret = ctx.eval_function(&ret);
    if ret.is_exception() {
      return Err(self.dump_error().into());
    }

    Ok(ret)
  }

  pub async fn run_event_loop(&mut self) -> Result<(), JsError> {
    poll_fn(|_cx| {
      self.perform_microtasks()?;
      self.check_promise_exceptions()?;
      return Poll::Ready(Ok(()));
    })
    .await
  }

  fn perform_microtasks(&mut self) -> Result<(), JsError> {
    loop {
      let has_microtask = self.js_runtime.execute_pending_job()?;
      if !has_microtask {
        break;
      }
    }

    Ok(())
  }

  fn check_promise_exceptions(&mut self) -> Result<(), JsError> {
    if let Some(e) = self.pending_promise_exceptions.first() {
      return Err(e.clone());
    }
    Ok(())
  }

  fn dump_error(&mut self) -> JsError {
    self.js_context.get_exception().into()
  }
}
