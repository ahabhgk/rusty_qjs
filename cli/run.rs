use crate::{
    error::{AnyError, QJS_ERROR},
    module::js_module_set_import_meta,
};
use libquickjs_sys as qjs;
use std::{
    cell::RefCell,
    env,
    ffi::CString,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

struct QtokRuntime {
    global_context: Rc<RefCell<qjs::JSContext>>,
    js_runtime: Rc<RefCell<qjs::JSRuntime>>,
}

// TODO: encapsulate runtime and context
impl Drop for QtokRuntime {
    fn drop(&mut self) {
        unsafe {
            qjs::JS_FreeRuntime(self.js_runtime.as_ptr());
            qjs::JS_FreeContext(self.global_context.as_ptr());
        }
    }
}

impl QtokRuntime {
    pub fn new() -> Self {
        let js_runtime = unsafe { qjs::JS_NewRuntime() };
        let global_context = unsafe { qjs::JS_NewContext(js_runtime) };
        let js_runtime = Rc::new(RefCell::new(unsafe { *js_runtime }));
        let global_context = Rc::new(RefCell::new(unsafe { *global_context }));
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

    pub fn eval_module(&self, path: &Path, is_main: bool) -> Result<(), AnyError> {
        let value = self.eval_file(path, is_main)?;
        unsafe {
            let ctx = self.global_context.as_ptr();
            qjs::JS_FreeValue(ctx, value);
        };
        Ok(())
    }

    fn eval_file(&self, path: &Path, is_main: bool) -> Result<qjs::JSValue, AnyError> {
        let code = fs::read_to_string(path)?;
        let code = &code[..];
        let name = path.to_str().unwrap();
        let ret = self.eval(code, name, true, true)?;
        let is_exception = unsafe { qjs::JS_IsException(ret) };
        if is_exception {
            return Err(AnyError::msg(QJS_ERROR));
        }
        js_module_set_import_meta(self.global_context.clone(), &ret, true, is_main)?;
        let ret = unsafe {
            let ctx = self.global_context.as_ptr();
            qjs::JS_EvalFunction(ctx, ret)
        };
        Ok(ret)
    }

    pub fn eval_script(&self, name: &str, code: &str) -> Result<qjs::JSValue, AnyError> {
        self.eval(code, name, false, false)
    }

    fn eval(
        &self,
        code: &str,
        name: &str,
        is_module: bool,
        compile_only: bool,
    ) -> Result<qjs::JSValue, AnyError> {
        let eval_flags = match (is_module, compile_only) {
            (true, true) => qjs::JS_EVAL_TYPE_MODULE | qjs::JS_EVAL_FLAG_COMPILE_ONLY,
            (true, false) => qjs::JS_EVAL_TYPE_MODULE,
            (false, true) => qjs::JS_EVAL_TYPE_GLOBAL | qjs::JS_EVAL_FLAG_COMPILE_ONLY,
            (false, false) => qjs::JS_EVAL_TYPE_GLOBAL,
        } as _;
        let ctx = self.global_context.as_ptr();
        let input = CString::new(code)?.as_ptr();
        let input_len = code.len() as _;
        let filename = CString::new(name)?.as_ptr();

        Ok(unsafe { qjs::JS_Eval(ctx, input, input_len, filename, eval_flags) })
    }

    pub fn run_event_loop(&self) -> Result<(), AnyError> {
        todo!();
        Ok(())
    }
}

pub fn run(script_path: PathBuf) -> Result<(), AnyError> {
    let script_path = env::current_dir()?.join(script_path);
    let qtok_runtime = QtokRuntime::new();
    qtok_runtime.eval_module(&script_path, true)?;
    // qtok_runtime.eval_script("<global>", "window.dispatchEvent(new Event('load'));")?;
    qtok_runtime.run_event_loop()?;
    // qtok_runtime.eval_script("<global>", "window.dispatchEvent(new Event('unload'));")?;
    Ok(())
}
