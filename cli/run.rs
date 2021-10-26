use crate::{error::AnyError, module::js_module_set_import_meta};
use futures::future::poll_fn;
use libquickjs_sys::JS_DupValue;
use rusty_qjs::{
  context::JsContext, error::JsError, runtime::JsRuntime, value::JsValue,
};
use std::{
  env,
  ffi::c_void,
  fs,
  path::{Path, PathBuf},
  ptr::NonNull,
  task::Poll,
};

// FIXME:
extern "C" fn host_promise_rejection_tracker(
  ctx: *mut libquickjs_sys::JSContext,
  _promise: libquickjs_sys::JSValue,
  reason: libquickjs_sys::JSValue,
  is_handled: ::std::os::raw::c_int,
  opaque: *mut ::std::os::raw::c_void,
) {
  if is_handled == 0 {
    let qtok = unsafe { &mut *(opaque as *mut Qtok) };
    unsafe { JS_DupValue(ctx, reason) };
    let ctx = NonNull::new(ctx).unwrap();
    let reason = JsValue { ctx, val: reason };
    qtok.pending_promise_exceptions.push(JsError::from(reason))
  }
}

struct Qtok {
  global_context: JsContext,
  js_runtime: JsRuntime,
  pending_promise_exceptions: Vec<JsError>,
  // pending_ops:
}

impl Qtok {
  pub fn new() -> Self {
    let mut js_runtime = JsRuntime::default();
    let global_context = JsContext::new(&mut js_runtime);
    let mut qtok = Self {
      global_context,
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
    let ctx = &mut self.global_context;
    let mut ret = ctx.eval(code, name, true, true)?;
    js_module_set_import_meta(ctx, &ret, true, is_main)?;
    // TODO: eval module, continue abstract eval?
    ret = ctx.eval_function(&ret);
    if ret.is_exception() {
      return Err(JsError::dump_from_context(ctx).into());
    }
    Ok(ret)
  }

  pub fn eval_script(
    &mut self,
    name: &str,
    code: &str,
  ) -> Result<JsValue, JsError> {
    self.global_context.eval(code, name, false, false)
  }

  pub async fn run_event_loop(&mut self) -> Result<(), JsError> {
    poll_fn(|cx| {
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
}

pub async fn run(script_path: PathBuf) -> Result<(), AnyError> {
  let script_path = env::current_dir()?.join(script_path);
  let mut qtok = Qtok::new();
  qtok.eval_module(&script_path, true)?;
  // qtok.eval_script("<global>", "window.dispatchEvent(new Event('load'));")?;
  qtok.run_event_loop().await?;
  // qtok.eval_script("<global>", "window.dispatchEvent(new Event('unload'));")?;
  Ok(())
}

#[cfg(test)]
mod tests {}
