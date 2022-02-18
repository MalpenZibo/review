use crate::body::BodyRewriter;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{visit_mut, Block, FnArg, Ident, Item, ItemFn, ReturnType, Type, Visibility};

pub(crate) struct Component {
    block: Box<Block>,
    props_type: Box<Type>,
    arg: FnArg,
    vis: Visibility,
    name: Ident,
    return_type: Box<Type>,
}

impl Parse for Component {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let parsed: Item = input.parse()?;

        match parsed {
            Item::Fn(func) => {
                let ItemFn {
                    vis, sig, block, ..
                } = func;

                if !sig.generics.params.is_empty() {
                    return Err(syn::Error::new_spanned(
                        sig.generics,
                        "function components can't contain generics",
                    ));
                }

                if sig.asyncness.is_some() {
                    return Err(syn::Error::new_spanned(
                        sig.asyncness,
                        "function components can't be async",
                    ));
                }

                if sig.constness.is_some() {
                    return Err(syn::Error::new_spanned(
                        sig.constness,
                        "const functions can't be function components",
                    ));
                }

                if sig.abi.is_some() {
                    return Err(syn::Error::new_spanned(
                        sig.abi,
                        "extern functions can't be function components",
                    ));
                }

                let return_type = match sig.output {
                    ReturnType::Default => {
                        return Err(syn::Error::new_spanned(
                            sig,
                            "function components must return `bom::VNode`",
                        ))
                    }
                    ReturnType::Type(_, ty) => ty,
                };

                let mut inputs = sig.inputs.into_iter();
                let arg: FnArg = inputs
                    .next()
                    .unwrap_or_else(|| syn::parse_quote! { _: &() });

                let ty = match &arg {
                    FnArg::Typed(arg) => match &*arg.ty {
                        Type::Reference(ty) => {
                            if ty.lifetime.is_some() {
                                return Err(syn::Error::new_spanned(
                                    &ty.lifetime,
                                    "reference must not have a lifetime",
                                ));
                            }

                            if ty.mutability.is_some() {
                                return Err(syn::Error::new_spanned(
                                    &ty.mutability,
                                    "reference must not be mutable",
                                ));
                            }

                            ty.elem.clone()
                        }
                        ty => {
                            let msg = format!(
                                "expected a reference to a `Properties` type (try: `&{}`)",
                                ty.to_token_stream()
                            );
                            return Err(syn::Error::new_spanned(ty, msg));
                        }
                    },

                    FnArg::Receiver(_) => {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "function components can't accept a receiver",
                        ));
                    }
                };

                // Checking after param parsing may make it a little inefficient
                // but that's a requirement for better error messages in case of receivers
                // `>0` because first one is already consumed.
                if inputs.len() > 0 {
                    let params: TokenStream = inputs.map(|it| it.to_token_stream()).collect();
                    return Err(syn::Error::new_spanned(
                        params,
                        "function components can accept at most one parameter for the props",
                    ));
                }

                Ok(Self {
                    props_type: ty,
                    block,
                    arg,
                    vis,
                    name: sig.ident,
                    return_type,
                })
            }
            item => Err(syn::Error::new_spanned(
                item,
                "`function_component` attribute can only be applied to functions",
            )),
        }
    }
}

pub(crate) struct ComponentName {
    pub(crate) component_name: Ident,
}

impl Parse for ComponentName {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(input.error("expected identifier for the component"));
        }

        let component_name = input.parse()?;

        Ok(Self { component_name })
    }
}

pub(crate) fn component_impl(
    name: ComponentName,
    component: Component,
) -> syn::Result<TokenStream> {
    let ComponentName { component_name } = name;

    let Component {
        mut block,
        props_type,
        arg,
        vis,
        name: function_name,
        return_type,
        ..
    } = component;

    if function_name == component_name {
        return Err(syn::Error::new_spanned(
            component_name,
            "the component must not have the same name as the function",
        ));
    }

    let ret_type = quote_spanned!(return_type.span()=> ::bom::VNode);
    let debug_name = format!("{:?}", component_name);

    let ctx_ident = Ident::new("context", Span::mixed_site());

    let mut body_rewriter = BodyRewriter::default();
    visit_mut::visit_block_mut(&mut body_rewriter, &mut *block);

    let quoted = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #vis struct #component_name(pub #props_type);

        impl ::std::fmt::Debug for #component_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(&#debug_name)
                 .finish()
            }
        }

        impl ::bom::ComponentProvider for #component_name {
            type Props = #props_type;

            fn run(#ctx_ident: &mut (::bom::FiberId, &mut ::bom::HookContext), #arg) -> #ret_type {
                #block
            }

            fn get_props<'a>(&'a self) -> &'a Self::Props {
                &self.0
            }
        }
    };

    Ok(quoted)
}
