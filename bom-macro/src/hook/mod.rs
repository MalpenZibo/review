use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::visit_mut;
use syn::{Ident, ItemFn, Signature};

mod body;

pub use body::BodyRewriter;

#[derive(Clone)]
pub struct HookFn {
    inner: ItemFn,
}

impl Parse for HookFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let func: ItemFn = input.parse()?;

        let sig = func.sig.clone();

        if sig.asyncness.is_some() {
            emit_error!(sig.asyncness, "async functions can't be hooks");
        }

        if sig.constness.is_some() {
            emit_error!(sig.constness, "const functions can't be hooks");
        }

        if sig.abi.is_some() {
            emit_error!(sig.abi, "extern functions can't be hooks");
        }

        if sig.unsafety.is_some() {
            emit_error!(sig.unsafety, "unsafe functions can't be hooks");
        }

        if !sig.ident.to_string().starts_with("use_") {
            emit_error!(sig.ident, "hooks must have a name starting with `use_`");
        }

        Ok(Self { inner: func })
    }
}

pub fn hook_impl(component: HookFn) -> syn::Result<TokenStream> {
    let HookFn { inner: original_fn } = component;

    let ItemFn {
        vis,
        sig,
        mut block,
        ..
    } = original_fn.clone();

    let Signature {
        ref fn_token,
        ref ident,
        ref inputs,
        output: ref hook_return_type,
        ref generics,
        ..
    } = sig;

    let (where_clause, ..) = generics.split_for_impl();

    let ctx_ident = Ident::new("context", Span::mixed_site());

    let mut body_rewriter = BodyRewriter::default();
    visit_mut::visit_block_mut(&mut body_rewriter, &mut *block);

    let some_inputs = inputs.len() > 0;

    let prev_inputs = if some_inputs {
        quote!( #inputs, )
    } else {
        quote!()
    };

    let output = quote! {
        #vis #fn_token #ident #generics ( #prev_inputs #ctx_ident: &mut (::bom::FiberId, &mut ::bom::HookContext)) #hook_return_type #where_clause #block
    };

    Ok(output)
}
