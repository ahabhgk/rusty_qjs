use std::fs;

fn main() {
  println!("cargo:rerun-if-changed=src/helper.c");
  build_quickjs();
}

fn build_quickjs() {
  let quickjs_version = fs::read_to_string("quickjs/VERSION")
    .expect("failed to read quickjs version");
  let quickjs_version = format!("\"{}\"", quickjs_version.trim());

  let mut config = cc::Build::new();

  config.include("quickjs/");

  config
    .file("quickjs/cutils.c")
    .file("quickjs/libbf.c")
    .file("quickjs/libregexp.c")
    .file("quickjs/libunicode.c")
    .file("quickjs/quickjs.c")
    // static functions and helper functions
    .file("src/helper.c");

  config
    .define("_GNU_SOURCE", None)
    .define("CONFIG_VERSION", quickjs_version.as_str())
    .define("CONFIG_BIGNUM", None)
    // The below flags are used by the official Makefile.
    .flag_if_supported("-Wchar-subscripts")
    .flag_if_supported("-Wno-array-bounds")
    .flag_if_supported("-Wno-format-truncation")
    .flag_if_supported("-Wno-missing-field-initializers")
    .flag_if_supported("-Wno-sign-compare")
    .flag_if_supported("-Wno-unused-parameter")
    .flag_if_supported("-Wundef")
    .flag_if_supported("-Wuninitialized")
    .flag_if_supported("-Wunused")
    .flag_if_supported("-Wwrite-strings")
    .flag_if_supported("-funsigned-char")
    // Below flags are added to supress warnings that appear on some
    // platforms.
    .flag_if_supported("-Wno-cast-function-type")
    .flag_if_supported("-Wno-implicit-fallthrough")
    .flag_if_supported("-Wno-enum-conversion")
    // cc uses the OPT_LEVEL env var by default, but we hardcode it to -O2
    // since release builds use -O3 which might be problematic for quickjs,
    // and debug builds only happen once anyway so the optimization slowdown
    // is fine.
    .opt_level(2);

  config.compile("libquickjs.a");
}
