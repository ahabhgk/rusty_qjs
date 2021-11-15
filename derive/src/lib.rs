use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
  fold::{fold_fn_arg, fold_signature, Fold},
  parse_macro_input, Block, FnArg, ItemFn, Signature, Visibility,
};

struct JsFunction {
  pub args: Vec<FnArg>,
  pub name: Option<Ident>,
  pub signature: Option<Signature>,
  pub signature_raw: Option<Signature>,
  pub block: Vec<Block>,
  pub visibility: Visibility,
}

impl JsFunction {
  pub fn new() -> Self {
    JsFunction {
      args: vec![],
      name: None,
      signature: None,
      signature_raw: None,
      visibility: Visibility::Inherited,
      block: vec![],
    }
  }
}

impl Fold for JsFunction {
  fn fold_fn_arg(&mut self, arg: FnArg) -> FnArg {
    self.args.push(arg.clone());
    fold_fn_arg(self, arg)
  }

  fn fold_signature(&mut self, signature: Signature) -> Signature {
    self.name = Some(format_ident!("{}", signature.ident));
    let mut new_signature = signature.clone();
    new_signature.ident =
      format_ident!("_generated_{}_generated_", signature.ident);
    self.signature = Some(new_signature);
    self.signature_raw = Some(signature.clone());
    fold_signature(self, signature)
  }

  fn fold_visibility(&mut self, v: Visibility) -> Visibility {
    self.visibility = v.clone();
    v
  }

  fn fold_block(&mut self, node: Block) -> Block {
    self.block.push(node.clone());
    node
  }
}

#[proc_macro_attribute]
pub fn js_function(_attr: TokenStream, input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as ItemFn);
  let mut js_fn = JsFunction::new();
  js_fn.fold_item_fn(input);
  let fn_name = js_fn.name.unwrap();
  let fn_block = js_fn.block;
  let signature = js_fn.signature.unwrap();
  let visibility = js_fn.visibility;
  let new_fn_name = signature.ident.clone();

  let expanded = quote! {
    #visibility extern "C" fn #fn_name(
      ctx: *mut rusty_qjs::sys::JSContext,
      this_val: rusty_qjs::sys::JSValue,
      argc: i32,
      argv: *mut rusty_qjs::sys::JSValue,
    ) -> rusty_qjs::sys::JSValue {
      use std::ptr;
      use std::panic::{self, AssertUnwindSafe};

      let mut ctx = JsContext::from_raw(ctx);
      let mut call_ctx = CallContext::new(&mut ctx, this_val, argc, argv);

      #[inline(always)]
      #signature #(#fn_block)*

      // TODO: catch_unwind
      let ret = #new_fn_name(call_ctx);
      let ret = ret.to_qjsrc();
      ret.raw_value
    }
  };
  expanded.into()
}

#[cfg(test)]
mod tests {}
