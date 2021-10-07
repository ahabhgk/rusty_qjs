mod module;
mod qjs_core;
mod run;

use clap::Clap;
use futures::FutureExt;
use qjs_core::error::AnyError;
use run::run;
use std::{future::Future, path::PathBuf, pin::Pin, process};
use tokio::{runtime, task};

#[derive(Clap, Debug)]
#[clap(name = "qtok")]
#[clap(author, about, version)]
enum Opts {
  /// Run a JavaScript program
  Run { script: PathBuf },
}

fn get_subcommand(
  opts: Opts,
) -> Pin<Box<dyn Future<Output = Result<(), AnyError>>>> {
  match opts {
    Opts::Run { script } => run(script).boxed_local(),
  }
}

fn main() {
  // let opts = Opts::parse();
  let opts = Opts::Run {
    script: PathBuf::from("cli/examples/mod.js"),
  };
  match run_local(get_subcommand(opts)) {
    Ok(_) => println!("run successed"),
    Err(e) => {
      eprintln!("[exception] {}", e);
      process::exit(1);
    }
  }
}

pub fn run_local<F, R>(future: F) -> R
where
  F: Future<Output = R>,
{
  let tokio_runtime = runtime::Builder::new_current_thread()
    .enable_io()
    .enable_time()
    .max_blocking_threads(32)
    .build()
    .unwrap();
  let local = task::LocalSet::new();
  local.block_on(&tokio_runtime, future)
}
