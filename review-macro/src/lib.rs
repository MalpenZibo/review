use crate::component::{component_impl, Component, ComponentName};
use crate::hook::{hook_impl, HookFn};
use syn::parse_macro_input;

mod body;
mod component;
mod hook;

#[proc_macro_attribute]
pub fn component(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as Component);
    let attr = parse_macro_input!(attr as ComponentName);

    component_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_error::proc_macro_error]
#[proc_macro_attribute]
pub fn hook(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as HookFn);

    if let Some(m) = proc_macro2::TokenStream::from(attr).into_iter().next() {
        return syn::Error::new_spanned(m, "hook attribute does not accept any arguments")
            .into_compile_error()
            .into();
    }

    hook_impl(item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
