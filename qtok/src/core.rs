use std::{ffi::c_void, fs, path::Path, task::Poll};

use futures::future::poll_fn;
use rusty_qjs::{
  Error, JSContext, JSRuntime, JSValue, OwnedJSContext, QuickjsRc,
};

use crate::{error::AnyError, ext};

extern "C" fn host_promise_rejection_tracker(
  ctx: *mut JSContext,
  _promise: JSValue,
  reason: JSValue,
  is_handled: ::std::os::raw::c_int,
  opaque: *mut ::std::os::raw::c_void,
) {
  if is_handled == 0 {
    let qtok = unsafe { &mut *(opaque as *mut Qtok) };
    let ctx = unsafe { ctx.as_mut() }.unwrap();
    reason.dup(ctx);
    // let reason = Local::from_qjsrc(ctx, reason);
    // qtok.pending_promise_exceptions.push(Error::from(reason))
    qtok.pending_promise_exceptions.push(reason.to_error(ctx))
  }
}

pub struct Qtok<'rt> {
  js_context: OwnedJSContext<'rt>,
  pending_promise_exceptions: Vec<Error>,
  // pending_ops:
}

// TODO: drop should called by js_context and js_runtime, hack for now
// impl Drop for Qtok<'_> {
//   fn drop(&mut self) {
//     self.js_context.free();
//     // self.js_runtime.free();
//   }
// }

impl<'rt> Qtok<'rt> {
  pub fn new(rt: &'rt mut JSRuntime) -> Self {
    let js_context = JSContext::new(rt);
    let mut qtok = Self {
      js_context,
      pending_promise_exceptions: Vec::new(),
    };
    // JS_SetMaxStackSize
    // JS_SetModuleLoaderFunc
    // JS_SetHostPromiseRejectionTracker
    let opaque = { &qtok as *const _ as *mut c_void };
    unsafe {
      qtok
        .js_context
        .get_runtime()
        .set_host_promise_rejection_tracker(
          Some(host_promise_rejection_tracker),
          opaque,
        )
    };
    // js_init_module_uv core, timers, error, fs, process...
    // tjs__bootstrap_globals fetch, url, performance, console, wasm...
    // tjs__add_builtins path, uuid, hashlib...
    ext::console::add_console(&mut *qtok.js_context).unwrap();

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
  ) -> Result<JSValue, AnyError> {
    let code = fs::read_to_string(path)?;
    let code = &code[..];
    let name = path.to_str().unwrap();

    let ret = self.js_context.compile_module(code, name);
    if ret.is_exception() {
      return Err(self.dump_error().into());
    }

    // js_module_set_import_meta(ctx, &ret, true, is_main)?;

    // TODO: eval module, continue abstract eval?
    let ret = self.js_context.eval_function(ret);
    if ret.is_exception() {
      return Err(self.dump_error().into());
    }

    Ok(ret)
  }

  fn compile_module(
    &mut self,
    code: &str,
    name: &str,
  ) -> Result<JSValue, Error> {
    let ret = self.js_context.compile_module(code, name);
    if ret.is_exception() {
      return Err(self.dump_error().into());
    }
    Ok(ret)
  }

  pub async fn run_event_loop(&mut self) -> Result<(), Error> {
    poll_fn(|_cx| {
      self.perform_microtasks()?;
      self.check_promise_exceptions()?;
      return Poll::Ready(Ok(()));
    })
    .await
  }

  fn perform_microtasks(&mut self) -> Result<(), Error> {
    loop {
      let has_microtask =
        self.js_context.get_runtime().execute_pending_job()?;
      if !has_microtask {
        break;
      }
    }

    Ok(())
  }

  fn check_promise_exceptions(&self) -> Result<(), Error> {
    if let Some(e) = self.pending_promise_exceptions.first() {
      return Err(e.clone());
    }
    Ok(())
  }

  fn dump_error(&mut self) -> Error {
    self
      .js_context
      .get_exception()
      .to_error(&mut self.js_context)
  }
}
