use crate::{
  module::js_module_set_import_meta,
  qjs_core::{
    context::JsContext,
    error::{AnyError, JsError},
    runtime::JsRuntime,
    value::JsValue,
  },
};
use futures::future::poll_fn;
use std::{
  env, fs,
  path::{Path, PathBuf},
  rc::Rc,
  task::Poll,
};

struct QtokRuntime {
  global_context: Rc<JsContext>,
  js_runtime: JsRuntime,
  // pending_ops:
}

impl QtokRuntime {
  pub fn new() -> Self {
    let js_runtime = JsRuntime::default();
    let global_context = JsContext::new(&js_runtime);
    // JS_SetMaxStackSize
    // JS_SetModuleLoaderFunc
    // JS_SetHostPromiseRejectionTracker
    // js_init_module_uv core, timers, error, fs, process...
    // tjs__bootstrap_globals fetch, url, performance, console, wasm...
    // tjs__add_builtins path, uuid, hashlib...
    Self {
      global_context,
      js_runtime,
    }
  }

  pub fn eval_module(
    &self,
    path: &Path,
    is_main: bool,
  ) -> Result<(), AnyError> {
    let _ = self.eval_file(path, is_main)?;
    Ok(())
  }

  fn eval_file(&self, path: &Path, is_main: bool) -> Result<JsValue, AnyError> {
    let code = fs::read_to_string(path)?;
    let code = &code[..];
    let name = path.to_str().unwrap();
    let ctx = &self.global_context;
    let mut ret = Rc::clone(ctx).eval(code, name, true, true)?;
    js_module_set_import_meta(Rc::clone(ctx), &ret, true, is_main)?;
    // TODO: eval module, continue abstract eval?
    ret = Rc::clone(ctx).eval_function(&ret);
    if ret.is_exception() {
      return Err(JsError::from_qjs_exception(Rc::clone(ctx), &ret).into());
    }
    Ok(ret)
  }

  pub fn eval_script(
    &self,
    name: &str,
    code: &str,
  ) -> Result<JsValue, AnyError> {
    Rc::clone(&self.global_context).eval(code, name, false, false)
  }

  // fn eval(
  //   &mut self,
  //   code: &str,
  //   name: &str,
  //   is_module: bool,
  //   compile_only: bool,
  // ) -> Result<qjs::JSValue, AnyError> {
  //   let eval_flags = match (is_module, compile_only) {
  //     (true, true) => qjs::JS_EVAL_TYPE_MODULE | qjs::JS_EVAL_FLAG_COMPILE_ONLY,
  //     (true, false) => qjs::JS_EVAL_TYPE_MODULE,
  //     (false, true) => {
  //       qjs::JS_EVAL_TYPE_GLOBAL | qjs::JS_EVAL_FLAG_COMPILE_ONLY
  //     }
  //     (false, false) => qjs::JS_EVAL_TYPE_GLOBAL,
  //   } as _;
  //   let ctx = self.global_context;
  //   let code_cstring = CString::new(code).unwrap();
  //   let input = code_cstring.as_ptr();
  //   let input_len = code.len() as _;
  //   let name_cstring = CString::new(name).unwrap();
  //   let filename = name_cstring.as_ptr();

  //   let value_raw =
  //     unsafe { qjs::JS_Eval(ctx, input, input_len, filename, eval_flags) };

  //   let is_exception = unsafe { qjs::JS_IsException(value_raw) };
  //   if is_exception {
  //     return Err(JsError::from_qjs_exception(ctx, value_raw).into());
  //   }
  //   Ok(value_raw)
  // }

  pub async fn run_event_loop(&self) -> Result<(), AnyError> {
    poll_fn(|cx| {
      self.perform_microtasks()?;
      return Poll::Ready(Ok(()));
    })
    .await
  }

  fn perform_microtasks(&self) -> Result<(), AnyError> {
    // loop {
    //   // qjs::JS_ExecutePendingJob(self.js_runtime, pctx)
    // }
    dbg!("looping!");

    Ok(())
  }
}

pub async fn run(script_path: PathBuf) -> Result<(), AnyError> {
  let script_path = env::current_dir()?.join(script_path);
  let qtok_runtime = QtokRuntime::new();
  qtok_runtime.eval_module(&script_path, true)?;
  // qtok_runtime.eval_script("<global>", "window.dispatchEvent(new Event('load'));")?;
  qtok_runtime.run_event_loop().await?;
  // qtok_runtime.eval_script("<global>", "window.dispatchEvent(new Event('unload'));")?;
  Ok(())
}

#[cfg(test)]
mod tests {}
