use crate::component::component_impl;
use crate::component::Component;
use crate::component::ComponentName;
use syn::parse_macro_input;

mod component;
mod component_body;

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

#[proc_macro]
pub fn make_answer(_item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}
