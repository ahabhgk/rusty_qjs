mod error;
mod module;
mod run;

use clap::Clap;
use run::run;
use std::{future::Future, path::PathBuf};
use tokio::{runtime, task};

#[derive(Clap, Debug)]
#[clap(name = "qtok")]
#[clap(author, about, version)]
enum Opts {
    /// Run a JavaScript program
    Run { script: PathBuf },
}

fn main() {
    let opts = Opts::parse();
    run_local(async move {
        match opts {
            Opts::Run { script } => run(script),
        }
    })
    .unwrap();
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
