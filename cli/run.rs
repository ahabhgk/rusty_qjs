use crate::error::AnyError;
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

impl QtokRuntime {
    pub fn new(is_main: bool) -> Self {
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

    pub fn eval_module(&self, path: &Path) {
        // let value =
    }

    fn eval_file(&self, path: &Path, is_main: bool) -> Result<(), AnyError> {
        let code = fs::read_to_string(path)?;
        let code = &code[..];
        let ctx = self.global_context.as_ptr();
        let ret = unsafe {
            let input = CString::new(code)?.as_ptr();
            let input_len = code.len() as _;
            let filename = CString::new(path.as_os_str().to_str().unwrap())?.as_ptr();
            let eval_flags = (qjs::JS_EVAL_FLAG_COMPILE_ONLY | qjs::JS_EVAL_TYPE_MODULE) as _;

            qjs::JS_Eval(ctx, input, input_len, filename, eval_flags)
        };
        let is_exception = unsafe { qjs::JS_IsException(ret) };
        if is_exception {
            return Err(AnyError::msg("TODO"));
        }
        // js_module_set_import_meta(ctx, ret, TRUE, is_main);
        // ret = JS_EvalFunction(ctx, ret);
        Ok(())
    }
}

fn eval_module() {}

pub fn run(script_path: PathBuf) -> Result<(), AnyError> {
    let script_path = env::current_dir()?.join(script_path);
    let code = fs::read_to_string(script_path)?;
    let qtok_runtime = QtokRuntime::new(true);
    Ok(())
}
