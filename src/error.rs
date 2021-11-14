use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
  #[error("Arguments index out of range")]
  ArgumentsIndexOutOfRange,
  #[error("{name}: {message}\n{stack}")]
  JSContextError {
    stack: String,
    message: String,
    name: String,
  },
}
