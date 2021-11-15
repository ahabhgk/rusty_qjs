# (WIP) rusty_qjs

Rust bindings to QuickJS.

## TODO

- [x] lifecycle for JsValue
  - [x] Local, Reference
  - [x] [set_property_str](https://github.com/ahabhgk/qtok.js/blob/caf3f0ae7bfeea26a2927e205d3ee9499bc5fe66/cli/src/ext/console.rs#L28)
  - [x] [global_object.free()](https://github.com/ahabhgk/qtok.js/blob/caf3f0ae7bfeea26a2927e205d3ee9499bc5fe66/cli/src/ext/console.rs#L30)
- [ ] JsFunction mapping
  - [ ] delete CallContext
- [ ] three-layer architecture
  - [ ] sys
  - [ ] safe fn with Reference type, QuickjsRc trait
  - [ ] OOP with Local type
