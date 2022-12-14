use proc_macro::{Diagnostic, Level};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{GenericParam, ItemTrait};

pub fn dyn_dyn_base(_args: TokenStream, mut input: ItemTrait) -> TokenStream {
    let vis = input.vis.clone();
    let ident = input.ident.clone();
    let generics = input.generics.clone();
    let (_, type_generics, where_clause) = input.generics.split_for_impl();

    let mut bad_spans = vec![];

    for generic_param in input.generics.params.iter() {
        if matches!(*generic_param, GenericParam::Lifetime(_)) {
            bad_spans.push(generic_param.span().unwrap());
        }
    }

    if !bad_spans.is_empty() {
        Diagnostic::spanned(
            bad_spans,
            Level::Error,
            "dyn-dyn base traits cannot have lifetime arguments",
        )
        .emit();
        return input.to_token_stream();
    }

    let base_trait_ident = format_ident!("__dyn_dyn_{}_Base", ident);
    let mut base_trait_impl_generics = generics.clone();

    base_trait_impl_generics.params.push(syn::parse2(
        quote!(__dyn_dyn_T: ?Sized + ::dyn_dyn::internal::DynDynImpl<dyn #ident #type_generics>)).unwrap()
    );

    let mut dyn_dyn_base_generics = generics.clone();
    dyn_dyn_base_generics
        .params
        .insert(0, syn::parse2(quote!('__dyn_dyn_lifetime)).unwrap());
    let (impl_generics, _, _) = dyn_dyn_base_generics.split_for_impl();

    input
        .supertraits
        .push(syn::parse2(quote!(#base_trait_ident #type_generics)).unwrap());

    let tokens = quote! {
        #input

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #vis unsafe trait #base_trait_ident #generics #where_clause {
            fn __dyn_dyn_get_table(&self) -> ::dyn_dyn::DynDynTable;
        }

        // SAFETY: This is just a straightforward passthrough, see the SAFETY comment for DynDynImpl impls in dyn_dyn_impl for more details
        unsafe impl #base_trait_impl_generics #base_trait_ident #type_generics for __dyn_dyn_T #where_clause {
            fn __dyn_dyn_get_table(&self) -> ::dyn_dyn::DynDynTable {
                <Self as ::dyn_dyn::internal::DynDynImpl<dyn #ident #type_generics>>::get_dyn_dyn_table(self)
            }
        }

        // SAFETY: This is just a straightforward passthrough, see the SAFETY comment for DynDynImpl impls in dyn_dyn_impl for more details
        unsafe impl #impl_generics ::dyn_dyn::DynDynBase for dyn #ident #type_generics + '__dyn_dyn_lifetime #where_clause {
            fn get_dyn_dyn_table(&self) -> ::dyn_dyn::DynDynTable {
                <Self as #base_trait_ident #type_generics>::__dyn_dyn_get_table(self)
            }
        }
    };

    tokens
}
