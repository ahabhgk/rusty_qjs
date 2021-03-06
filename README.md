# (WIP) rusty_qjs

Rust bindings to QuickJS.

## Todo

- [ ] JSValue::new_... returns a Result
- [ ] specific JSValue type, like JsString, JsNumber...?
- [ ] catch unwind for extern "C" fn?

## Feature

### local

The JSValue of QuickJS is using reference counting to manage the memory. So we
create a Local handle to help you free the JSValue. The Local will call
JS_FreeValue when it drops. You can simply use `to_local` to convert a JSValue
into a Local handle, then enjoy the conveniences of it.

## Example

```rust
use rusty_qjs::{JSContext, JSRuntime, JSValue};
use std::io::Write;

fn js_print(ctx: &mut JSContext, _this: JSValue, argv: &[JSValue]) -> JSValue {
  let output = argv
    .iter()
    .map(|value| value.to_rust_string(ctx))
    .collect::<Vec<String>>()
    .join(" ");
  let mut stdout = std::io::stdout();
  stdout.write_all(output.as_bytes()).unwrap();
  stdout.write_all(b"\n").unwrap();
  JSValue::new_undefined()
}

fn setup_console(ctx: &mut JSContext) {
  let global = ctx.get_global_object().to_local(ctx);
  let console = JSValue::new_object(ctx).to_local(ctx);
  let log = JSValue::new_function(ctx, js_print, "log", 1).to_local(ctx);

  console.set_property_str("log", log).unwrap();
  global.set_property_str("console", console).unwrap();
}

fn main() {
  let rt = &mut JSRuntime::new();
  let ctx = &mut JSContext::new(rt);

  setup_console(ctx);
  ctx.eval_script("console.log(\"hello world\")", "<test>");
}
```

For a more in-depth example, look at
[qtok](https://github.com/ahabhgk/rusty_qjs/tree/main/qtok)
