itest!(should_throw_error {
  args: "run 1_throw_error.js",
  output: "1_throw_error.js.out",
  exit_code: 1,
});

itest!(should_throw_promise_rejection {
  args: "run 2_throw_promise_rejection.js",
  output: "2_throw_promise_rejection.js.out",
  exit_code: 1,
});
