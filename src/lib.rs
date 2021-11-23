#![deny(missing_docs)]

//! Rust bindings to QuickJS.

mod context;
pub mod error;
#[cfg(feature = "local")]
mod local;
mod quickjs_rc;
mod runtime;
mod support;
mod value;

pub use context::JSContext;
pub use context::OwnedJSContext;
#[cfg(feature = "local")]
pub use local::Local;
pub use quickjs_rc::QuickjsRc;
pub use runtime::JSRuntime;
pub use runtime::OwnedJSRuntime;
pub use value::JSCFunction;
pub use value::JSValue;
