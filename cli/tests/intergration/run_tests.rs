itest!(should_throw_error {
  args: "run 1_throw_error.js",
  output: "1_throw_error.js.out",
  exit_code: 1,
});
