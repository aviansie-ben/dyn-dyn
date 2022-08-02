use proc_macro::{Diagnostic, Level};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{GenericParam, ItemTrait};
use syn::spanned::Spanned;

use crate::util;

pub fn dyn_dyn_base(_args: TokenStream, mut input: ItemTrait) -> TokenStream {
    let dyn_dyn = util::dyn_dyn_crate();
    let vis = input.vis.clone();
    let ident = input.ident.clone();
    let generics = input.generics.clone();
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let mut bad_spans = vec![];

    for generic_param in input.generics.params.iter() {
        if matches!(*generic_param, GenericParam::Lifetime(_)) {
            bad_spans.push(generic_param.span().unwrap());
        }
    }

    if !bad_spans.is_empty() {
        Diagnostic::spanned(bad_spans, Level::Error, "dyn-dyn base traits cannot have lifetime arguments").emit();
        return input.to_token_stream();
    }

    let marker_ident = format_ident!("__dyn_dyn_{}_Marker", ident);

    input.supertraits.push(syn::parse2(quote!(#dyn_dyn::internal::DynDynBase<#marker_ident #type_generics>)).unwrap());

    let marker_contents = input.generics.params.iter().filter_map(|p| {
        match *p {
            GenericParam::Type(ref p) => Some(p.ident.clone()),
            _ => None
        }
    });
    let marker_contents = quote!(#(#marker_contents),*);

    let tokens = quote! {
        #input

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #vis struct #marker_ident #generics(#marker_contents) #where_clause;

        impl #impl_generics #dyn_dyn::internal::DynDynImpl<dyn #ident #type_generics> for dyn #ident #type_generics #where_clause {
            type BaseMarker = #marker_ident #type_generics;

            const IS_SEND: bool = false;
            const IS_SYNC: bool = false;

            fn get_dyn_dyn_table(&self) -> #dyn_dyn::DynDynTable { <Self as #dyn_dyn::internal::DynDynBase<Self::BaseMarker>>::get_dyn_dyn_table(self) }
        }
    };

    tokens
}
