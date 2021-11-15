use std::{
  ops::{Deref, DerefMut},
  ptr::NonNull,
};

use crate::support::Opaque;

extern "C" {
  pub fn JS_NewRuntime() -> *mut JSRuntime;
}

// use crate::{sys, Error, JsContext};

// use std::{ffi::c_void, ptr};

// #[derive(Debug)]
// pub struct JsRuntime {
//   pub raw_runtime: *mut sys::JSRuntime,
// }

// impl Default for JsRuntime {
//   fn default() -> Self {
//     let raw_runtime = unsafe { sys::JS_NewRuntime() };
//     Self { raw_runtime }
//   }
// }

// // impl Drop for JsRuntime {
// //   fn drop(&mut self) {
// //     unsafe { libquickjs_sys::JS_FreeRuntime(self.raw_runtime) };
// //   }
// // }

// impl JsRuntime {
//   /// Set callback to handle host promise rejection
//   ///
//   /// # Safety
//   ///
//   /// TODO
//   pub unsafe fn set_host_promise_rejection_tracker(
//     &self,
//     tracker: sys::JSHostPromiseRejectionTracker,
//     opaque: *mut c_void,
//   ) {
//     sys::JS_SetHostPromiseRejectionTracker(self.raw_runtime, tracker, opaque);
//   }

//   pub fn execute_pending_job(&self) -> Result<bool, Error> {
//     let pctx = &mut ptr::null_mut();
//     let res = unsafe { sys::JS_ExecutePendingJob(self.raw_runtime, pctx) };
//     match res {
//       0 => Ok(false),
//       1 => Ok(true),
//       2.. => panic!("JS_ExecutePendingJob never return >1"),
//       _ => Err(JsContext::from_raw(*pctx).get_exception().into()),
//     }
//   }

//   pub fn free(&mut self) {
//     unsafe { sys::JS_FreeRuntime(self.raw_runtime) };
//   }
// }

#[repr(C)]
#[derive(Debug, Copy, Clone)] // Clone?
pub struct JSRuntime(Opaque);

impl JSRuntime {
  pub fn new() -> OwnedJSRuntime {
    let rt = unsafe { JS_NewRuntime() };
    let rt = NonNull::new(rt).unwrap();
    OwnedJSRuntime(rt)
  }
}

pub struct OwnedJSRuntime(NonNull<JSRuntime>);

impl Deref for OwnedJSRuntime {
  type Target = JSRuntime;
  fn deref(&self) -> &Self::Target {
    unsafe { self.0.as_ref() }
  }
}

impl DerefMut for OwnedJSRuntime {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.0.as_mut() }
  }
}
