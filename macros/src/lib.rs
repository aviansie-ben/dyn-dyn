//! This crate provides procedural macros meant to be used with the `dyn-dyn` crate. This crate should not be depended upon directly:
//! instead, the versions of these macros re-exported from the `dyn-dyn` crate itself should be used.

#![forbid(unsafe_code)] // We generate unsafe code, but don't want to accidentally use unsafe code at compile-time
#![allow(clippy::needless_borrowed_reference)]
#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]

extern crate proc_macro;

use crate::cast::DynDynCastInput;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, ItemImpl, ItemTrait, Token, Type};

mod base;
mod cast;
mod impl_block;

#[proc_macro]
pub fn dyn_dyn_cast(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    cast::dyn_dyn_cast(parse_macro_input!(input as DynDynCastInput)).into()
}

#[proc_macro_attribute]
pub fn dyn_dyn_base(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    base::dyn_dyn_base(args.into(), parse_macro_input!(input as ItemTrait)).into()
}

struct DerivedTypes(Punctuated<Type, Token![,]>);

impl Parse for DerivedTypes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DerivedTypes(Punctuated::parse_terminated(input)?))
    }
}

#[proc_macro_attribute]
pub fn dyn_dyn_impl(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    impl_block::dyn_dyn_impl(
        parse_macro_input!(args as DerivedTypes).0,
        parse_macro_input!(input as ItemImpl),
    )
    .into()
}
