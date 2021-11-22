use crate::JSContext;

/// JSValue is using reference counting, so it is important
/// to explicitly duplicate or free JSValues.
pub trait QuickjsRc {
  /// Decrement the reference count.
  fn free(&mut self, ctx: &mut JSContext);

  /// Increment the reference count.
  fn dup(&self, ctx: &mut JSContext) -> Self;
}
