use proc_macro::{Diagnostic, Level};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::{GenericParam, ItemTrait};

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

    base_trait_impl_generics.params.push(syn::parse2(quote!(__dyn_dyn_T: ?Sized + #dyn_dyn::internal::DynDynDerived<dyn #ident #type_generics>)).unwrap());

    input
        .supertraits
        .push(syn::parse2(quote!(#base_trait_ident #type_generics)).unwrap());

    let tokens = quote! {
        #input

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        #vis unsafe trait #base_trait_ident #generics #where_clause {
            fn __dyn_dyn_get_table(&self) -> #dyn_dyn::DynDynTable;
        }

        unsafe impl #base_trait_impl_generics #base_trait_ident #type_generics for __dyn_dyn_T #where_clause {
            fn __dyn_dyn_get_table(&self) -> #dyn_dyn::DynDynTable { <Self as #dyn_dyn::internal::DynDynDerived<dyn #ident #type_generics>>::get_dyn_dyn_table(self) }
        }

        impl #impl_generics #dyn_dyn::internal::DynDynImpl for dyn #ident #type_generics #where_clause {
            type BaseDynDyn = dyn #ident #type_generics;

            const IS_SEND: bool = false;
            const IS_SYNC: bool = false;

            fn get_dyn_dyn_table(&self) -> #dyn_dyn::DynDynTable { <Self as #base_trait_ident #type_generics>::__dyn_dyn_get_table(self) }
        }

        impl #impl_generics #dyn_dyn::internal::DynDynImpl for dyn #ident #type_generics + Send #where_clause {
            type BaseDynDyn = dyn #ident #type_generics;

            const IS_SEND: bool = true;
            const IS_SYNC: bool = false;

            fn get_dyn_dyn_table(&self) -> #dyn_dyn::DynDynTable { <Self as #base_trait_ident #type_generics>::__dyn_dyn_get_table(self) }
        }

        impl #impl_generics #dyn_dyn::internal::DynDynImpl for dyn #ident #type_generics + Sync #where_clause {
            type BaseDynDyn = dyn #ident #type_generics;

            const IS_SEND: bool = false;
            const IS_SYNC: bool = true;

            fn get_dyn_dyn_table(&self) -> #dyn_dyn::DynDynTable { <Self as #base_trait_ident #type_generics>::__dyn_dyn_get_table(self) }
        }

        impl #impl_generics #dyn_dyn::internal::DynDynImpl for dyn #ident #type_generics + Send + Sync #where_clause {
            type BaseDynDyn = dyn #ident #type_generics;

            const IS_SEND: bool = true;
            const IS_SYNC: bool = true;

            fn get_dyn_dyn_table(&self) -> #dyn_dyn::DynDynTable { <Self as #base_trait_ident #type_generics>::__dyn_dyn_get_table(self) }
        }
    };

    tokens
}
