pub mod runtime;
pub mod context;
pub mod value;
pub mod atom;

#[cfg(test)]
mod tests {
  #[test]
  fn it_works() {
    assert_eq!(2 + 2, 4);
  }
}
