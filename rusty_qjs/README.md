# rusty_qjs

Safe abstraction for libquickjs_sys.

## TODO

```rust
#[derive(Debug)]
pub struct JsRuntime(pub NonNull<JSRuntime>);

#[derive(Debug)]
pub struct JsContext(pub NonNull<JSContext>);

#[derive(Debug, ...)]
pub struct JsValue {
  value: JSValue,
  ctx: NonNull<JSContext>,
}

#[derive(Debug, ...)]
pub struct JsAtom {
  atom: JSAtom,
  ctx: NonNull<JSContext>,
}
```

## data type

// DRAFT: 子类型多态？

```text
JsValue
├── JsModule
├── JsError
├── JsFunction
└── ...
```

## libquickjs_sys methods

- JS_ExecutePendingJob: return < 0 if exception, 0 if no job pending, 1 if a job was
executed successfully. the context of the job is stored in '*pctx'
- JS_ToCStringLen2: return (NULL, 0) if exception. return pointer into a JSString with a live ref_count cesu8 determines if non-BMP1 codepoints are encoded as 1 or 2 utf-8 sequences
