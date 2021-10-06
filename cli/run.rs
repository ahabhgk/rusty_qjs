use crate::{
    error::{AnyError, QJS_ERROR},
    module::js_module_set_import_meta,
};
use futures::future::poll_fn;
use libquickjs_sys as qjs;
use std::{
    env,
    ffi::CString,
    fs,
    path::{Path, PathBuf},
    task::Poll,
};

struct QtokRuntime {
    global_context: *mut qjs::JSContext,
    js_runtime: *mut qjs::JSRuntime,
}

// TODO: encapsulate runtime and context
impl Drop for QtokRuntime {
    fn drop(&mut self) {
        unsafe {
            qjs::JS_FreeContext(self.global_context);
            qjs::JS_FreeRuntime(self.js_runtime);
        }
    }
}

impl QtokRuntime {
    pub fn new() -> Self {
        let js_runtime = unsafe { qjs::JS_NewRuntime() };
        let global_context = unsafe { qjs::JS_NewContext(js_runtime) };
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

    pub fn eval_module(&mut self, path: &Path, is_main: bool) -> Result<(), AnyError> {
        let value = self.eval_file(path, is_main)?;
        unsafe { qjs::JS_FreeValue(self.global_context, value) };
        Ok(())
    }

    fn eval_file(&mut self, path: &Path, is_main: bool) -> Result<qjs::JSValue, AnyError> {
        let code = fs::read_to_string(path)?;
        let code = &code[..];
        let name = path.to_str().unwrap();
        let ret = self.eval(code, name, true, true);
        let is_exception = unsafe { qjs::JS_IsException(ret) };
        if is_exception {
            return Err(AnyError::msg(QJS_ERROR));
        }
        js_module_set_import_meta(self.global_context, &ret, true, is_main)?;
        let ret = unsafe { qjs::JS_EvalFunction(self.global_context, ret) };
        Ok(ret)
    }

    pub fn eval_script(&mut self, name: &str, code: &str) -> qjs::JSValue {
        self.eval(code, name, false, false)
    }

    fn eval(
        &mut self,
        code: &str,
        name: &str,
        is_module: bool,
        compile_only: bool,
    ) -> qjs::JSValue {
        let eval_flags = match (is_module, compile_only) {
            (true, true) => qjs::JS_EVAL_TYPE_MODULE | qjs::JS_EVAL_FLAG_COMPILE_ONLY,
            (true, false) => qjs::JS_EVAL_TYPE_MODULE,
            (false, true) => qjs::JS_EVAL_TYPE_GLOBAL | qjs::JS_EVAL_FLAG_COMPILE_ONLY,
            (false, false) => qjs::JS_EVAL_TYPE_GLOBAL,
        } as _;
        let code_cstring = CString::new(code).unwrap();
        let input = code_cstring.as_ptr();
        let input_len = code.len() as _;
        let name_cstring = CString::new(name).unwrap();
        let filename = name_cstring.as_ptr();

        let value_raw =
            unsafe { qjs::JS_Eval(self.global_context, input, input_len, filename, eval_flags) };
        value_raw
    }

    pub async fn run_event_loop(&self) -> Result<(), AnyError> {
        poll_fn(|cx| {
            return Poll::Ready(Ok(()));
        })
        .await
    }
}

pub async fn run(script_path: PathBuf) -> Result<(), AnyError> {
    let script_path = env::current_dir()?.join(script_path);
    let mut qtok_runtime = QtokRuntime::new();
    qtok_runtime.eval_module(&script_path, true)?;
    // qtok_runtime.eval_script("<global>", "window.dispatchEvent(new Event('load'));")?;
    qtok_runtime.run_event_loop().await?;
    // qtok_runtime.eval_script("<global>", "window.dispatchEvent(new Event('unload'));")?;
    Ok(())
}

#[cfg(test)]
mod tests {}
