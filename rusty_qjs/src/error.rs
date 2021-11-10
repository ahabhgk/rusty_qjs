use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error("Arguments index out of range")]
  ArgumentsIndexOutOfRange,
}
