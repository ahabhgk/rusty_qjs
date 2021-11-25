use crate::{JSContext, JSRuntime};

/// JSValue is using reference counting, so it is important
/// to explicitly duplicate or free JSValues.
pub trait QuickjsRc {
  /// Decrement the reference count.
  fn free(&mut self, ctx: &mut JSContext);

  /// Decrement the reference count by runtime.
  fn free_runtime(&mut self, rt: &mut JSRuntime);

  /// Increment the reference count.
  fn dup(&mut self, ctx: &mut JSContext) -> Self;

  /// Increment the reference count by runtime.
  fn dup_runtime(&mut self, rt: &mut JSRuntime) -> Self;
}
