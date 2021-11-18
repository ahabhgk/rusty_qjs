use crate::JSContext;

pub trait QuickjsRc {
  fn free(&mut self, ctx: &mut JSContext);

  fn dup(&self, ctx: &mut JSContext) -> Self;
}
