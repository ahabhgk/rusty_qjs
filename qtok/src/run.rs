use std::{env, path::PathBuf};

use rusty_qjs::JSRuntime;

use crate::{core::Qtok, error::AnyError};

pub async fn run(script_path: PathBuf) -> Result<(), AnyError> {
  let rt = &mut JSRuntime::new();
  let mut qtok = Qtok::new(rt);

  let script_path = env::current_dir()?.join(script_path);

  qtok.eval_module(&script_path, true)?;
  // qtok.eval_script("<global>", "window.dispatchEvent(new Event('load'));")?;
  qtok.run_event_loop().await?;
  // qtok.eval_script("<global>", "window.dispatchEvent(new Event('unload'));")?;
  Ok(())
}

#[cfg(test)]
mod tests {}
