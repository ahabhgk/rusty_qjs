use super::{context::JsContext, value::JsValue};
use libquickjs_sys as qjs;
use std::{error::Error, fmt::Display, rc::Rc};

#[derive(Debug, Clone)]
pub struct JsError {
  stack: String,
  message: String,
  name: String,
}

impl From<Rc<JsContext>> for JsError {
  fn from(ctx: Rc<JsContext>) -> Self {
    let value = unsafe { qjs::JS_GetException(Rc::clone(&ctx).inner()) };
    let value = JsValue::from_qjs(Rc::clone(&ctx), value);

    let (name, message, stack) = if value.is_error(Rc::clone(&ctx)) {
      let name = match value.get_property(Rc::clone(&ctx), "name") {
        Some(v) => v.into(),
        None => "Error".to_owned(),
      };

      let message = match value.get_property(Rc::clone(&ctx), "message") {
        Some(v) => v.into(),
        None => "".to_owned(),
      };

      let stack = match value.get_property(Rc::clone(&ctx), "stack") {
        Some(v) => v.into(),
        None => "".to_owned(),
      };

      (name, message, stack)
    } else {
      let message = value.clone().into();
      ("Exception".to_owned(), message, "".to_owned())
    };

    Self {
      name,
      stack,
      message,
    }
  }
}

// impl JsError {
//   pub fn from_qjs_exception(ctx: Rc<JsContext>, value: &JsValue) -> Self {
//     if !value.is_exception() {
//       panic!("value is not an exception");
//     }

//     let value = unsafe { qjs::JS_GetException(Rc::clone(&ctx).inner()) };
//     let value = JsValue::from_qjs(Rc::clone(&ctx), value);

//     let (name, message, stack) = if value.is_error(Rc::clone(&ctx)) {
//       let name = match value.get_property(Rc::clone(&ctx), "name") {
//         Some(v) => v.into(),
//         None => "Error".to_owned(),
//       };

//       let message = match value.get_property(Rc::clone(&ctx), "message") {
//         Some(v) => v.into(),
//         None => "".to_owned(),
//       };

//       let stack = match value.get_property(Rc::clone(&ctx), "stack") {
//         Some(v) => v.into(),
//         None => "".to_owned(),
//       };

//       (name, message, stack)
//     } else {
//       let message = value.clone().into();
//       ("Exception".to_owned(), message, "".to_owned())
//     };

//     Self {
//       name,
//       stack,
//       message,
//     }
//   }
// }

impl Error for JsError {}

impl Display for JsError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}\n{}", self.name, self.message, self.stack)
  }
}

// impl TryFrom<qjs::JSValue> for JsError {
//     type Error = AnyError;

//     fn try_from(value: qjs::JSValue) -> Result<Self, Self::Error> {
//         let is_exception = unsafe { qjs::JS_IsException(value) };
//         if is_exception {
//             return Err(AnyError::msg("is not an error"));
//         }
//         Ok(Self::from(value))
//     }
// }
