#![feature(proc_macro_diagnostic)]

extern crate proc_macro;

use syn::{ItemImpl, ItemTrait, Token, Type, parse_macro_input};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

mod base;
mod derived;
mod util;

#[proc_macro_attribute]
pub fn dyn_dyn_base(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    base::dyn_dyn_base(args.into(), parse_macro_input!(input as ItemTrait)).into()
}

struct DerivedTypes(Punctuated<Type, Token![,]>);

impl Parse for DerivedTypes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DerivedTypes(Punctuated::parse_terminated(input)?))
    }
}

#[proc_macro_attribute]
pub fn dyn_dyn_derived(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derived::dyn_dyn_derived(parse_macro_input!(args as DerivedTypes).0, parse_macro_input!(input as ItemImpl)).into()
}
