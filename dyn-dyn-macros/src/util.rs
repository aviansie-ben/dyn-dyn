use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;

pub fn dyn_dyn_crate() -> TokenStream {
    match crate_name("dyn-dyn").expect("dyn-dyn must be a dependency for dyn-dyn-macros to work") {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
    }
}
