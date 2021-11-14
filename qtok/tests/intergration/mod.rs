use os_pipe::pipe;
use std::{
  io::Read,
  path::PathBuf,
  process::{Command, Stdio},
};

#[macro_export]
macro_rules! itest(
($name:ident {$( $key:ident: $value:expr,)*}) => {
  #[test]
  fn $name() {
    ($crate::intergration::CheckOutputIntegrationTest {
      $(
        $key: $value,
       )*
      .. Default::default()
    }).run()
  }
}
);

/// from: https://github.com/denoland/deno/blob/a632dc5b0d321e704bc56e2ffd4582494c3efbea/test_util/src/lib.rs#L1651
#[derive(Default)]
pub struct CheckOutputIntegrationTest {
  args: &'static str,
  output: &'static str,
  exit_code: i32,
  output_str: Option<&'static str>,
}

impl CheckOutputIntegrationTest {
  fn run(&self) {
    let args = self.args.split_whitespace();
    let qtok_exe = qtok_exe_path();
    println!("qtok_exe path {}", qtok_exe.display());

    let (mut reader, writer) = pipe().unwrap();
    let testdata_dir = testdata_path();
    let mut command = Command::new(qtok_exe);
    println!("qtok_exe args {}", self.args);
    println!("qtok_exe testdata path {:?}", &testdata_dir);
    command.args(args);
    command.current_dir(&testdata_dir);
    command.stdin(Stdio::piped());
    let writer_clone = writer.try_clone().unwrap();
    command.stderr(writer_clone);
    command.stdout(writer);

    let mut process = command.spawn().expect("failed to execute process");

    // Very important when using pipes: This parent process is still
    // holding its copies of the write ends, and we have to close them
    // before we read, otherwise the read end will never report EOF. The
    // Command object owns the writers now, and dropping it closes them.
    drop(command);

    let mut actual = String::new();
    reader.read_to_string(&mut actual).unwrap();

    let status = process.wait().expect("failed to finish process");

    if let Some(exit_code) = status.code() {
      if self.exit_code != exit_code {
        println!("OUTPUT\n{}\nOUTPUT", actual);
        panic!(
          "bad exit code, expected: {:?}, actual: {:?}",
          self.exit_code, exit_code
        );
      }
    } else {
      #[cfg(unix)]
      {
        use std::os::unix::process::ExitStatusExt;
        let signal = status.signal().unwrap();
        println!("OUTPUT\n{}\nOUTPUT", actual);
        panic!(
          "process terminated by signal, expected exit code: {:?}, actual signal: {:?}",
          self.exit_code, signal
        );
      }
      #[cfg(not(unix))]
      {
        println!("OUTPUT\n{}\nOUTPUT", actual);
        panic!("process terminated without status code on non unix platform, expected exit code: {:?}", self.exit_code);
      }
    }

    let expected = if let Some(s) = self.output_str {
      s.to_owned()
    } else {
      let output_path = testdata_dir.join(self.output);
      println!("output path {}", output_path.display());
      std::fs::read_to_string(output_path).expect("cannot read output")
    };

    if !wildcard_match(&expected, &actual) {
      println!("OUTPUT\n{}\nOUTPUT", actual);
      println!("EXPECTED\n{}\nEXPECTED", expected);
      panic!("pattern match failed");
    }
  }
}

pub fn wildcard_match(pattern: &str, s: &str) -> bool {
  pattern_match(pattern, s, "[WILDCARD]")
}

pub fn pattern_match(pattern: &str, s: &str, wildcard: &str) -> bool {
  // Normalize line endings
  let mut s = s.replace("\r\n", "\n");
  let pattern = pattern.replace("\r\n", "\n");

  if pattern == wildcard {
    return true;
  }

  let parts = pattern.split(wildcard).collect::<Vec<&str>>();
  if parts.len() == 1 {
    return pattern == s;
  }

  if !s.starts_with(parts[0]) {
    return false;
  }

  // If the first line of the pattern is just a wildcard the newline character
  // needs to be pre-pended so it can safely match anything or nothing and
  // continue matching.
  if pattern.lines().next() == Some(wildcard) {
    s.insert(0, '\n');
  }

  let mut t = s.split_at(parts[0].len());

  for (i, part) in parts.iter().enumerate() {
    if i == 0 {
      continue;
    }
    dbg!(part, i);
    if i == parts.len() - 1 && (part.is_empty() || *part == "\n") {
      dbg!("exit 1 true", i);
      return true;
    }
    if let Some(found) = t.1.find(*part) {
      dbg!("found ", found);
      t = t.1.split_at(found + part.len());
    } else {
      dbg!("exit false ", i);
      return false;
    }
  }

  dbg!("end ", t.1.len());
  t.1.is_empty()
}

fn target_dir() -> PathBuf {
  let current_exe = std::env::current_exe().unwrap();
  let target_dir = current_exe.parent().unwrap().parent().unwrap();
  target_dir.into()
}

fn qtok_exe_path() -> PathBuf {
  // Something like /Users/ahabhgk/qtok/target/debug/deps/qtok
  let mut p = target_dir().join("qtok");
  if cfg!(windows) {
    p.set_extension("exe");
  }
  p
}

fn root_path() -> PathBuf {
  PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR")))
    .parent()
    .unwrap()
    .to_path_buf()
}

fn tests_path() -> PathBuf {
  root_path().join("cli").join("tests")
}

fn testdata_path() -> PathBuf {
  tests_path().join("testdata")
}

mod run_tests;
