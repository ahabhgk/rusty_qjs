use crate::qjs::Context;

pub trait Extension {
    fn apply(&self, context: &mut Context);
}
