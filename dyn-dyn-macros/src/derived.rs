use proc_macro::{Diagnostic, Level};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{GenericParam, ItemImpl, Token, Type};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use crate::util;

pub fn dyn_dyn_derived(args: Punctuated<Type, Token![,]>, input: ItemImpl) -> TokenStream {
    if input.trait_.is_none() {
        Diagnostic::spanned(proc_macro::Span::call_site(), Level::Error, "Cannot use dyn_dyn_derived on an inherent impl block").emit();
        return input.to_token_stream();
    } else if input.trait_.as_ref().unwrap().0.is_some() {
        Diagnostic::spanned(proc_macro::Span::call_site(), Level::Error, "Cannot use dyn_dyn_derived on a negative impl block").emit();
        return input.to_token_stream();
    }

    let dyn_dyn = util::dyn_dyn_crate();
    let self_ty = input.self_ty.clone();
    let trait_ = input.trait_.clone().unwrap().1;
    let generics = input.generics.clone();
    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    let mut bad_spans = vec![];

    for generic_param in input.generics.params.iter() {
        if matches!(*generic_param, GenericParam::Lifetime(_)) {
            bad_spans.push(generic_param.span().unwrap());
        }
    }

    if !bad_spans.is_empty() {
        Diagnostic::spanned(bad_spans, Level::Error, "dyn-dyn implementors cannot have lifetime arguments").emit();
        return input.to_token_stream();
    }

    let table_ident = format_ident!("__dyn_dyn_{}_DynTable", self_ty.span().start().line);
    let num_table_entries = args.len();

    let turbo_tok = if input.generics.params.is_empty() {
        None
    } else {
        Some(quote!(::))
    };

    let convert_fns_0 = (0..num_table_entries).map(|i| format_ident!("__convert_{}", i));
    let convert_tys_0 = args.iter();
    let convert_fns_1 = convert_fns_0.clone();
    let convert_tys_1 = args.iter();

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
        pub struct #table_ident #generics(#marker_contents) #where_clause;

        impl #impl_generics #table_ident #type_generics #where_clause {
            #(
                const fn #convert_fns_0(p: *const #self_ty) -> *const dyn #convert_tys_0 {
                    p as *const dyn #convert_tys_0
                }
            )*

            pub const __TABLE: [#dyn_dyn::DynDynTableEntry; #num_table_entries] = unsafe { [
                #(
                    #dyn_dyn::DynDynTableEntry::new::<
                        #self_ty,
                        dyn #convert_tys_1,
                        dyn #convert_tys_1 + Send,
                        dyn #convert_tys_1 + Sync,
                        dyn #convert_tys_1 + Send + Sync,
                        _
                    >(Self::#convert_fns_1)
                ),*
            ] };
        }

        unsafe impl #impl_generics #dyn_dyn::internal::DynDynBase<<dyn #trait_ as #dyn_dyn::internal::DynDynImpl<dyn #trait_>>::BaseMarker> for #self_ty #where_clause {
            fn get_dyn_dyn_table(&self) -> #dyn_dyn::DynDynTable {
                #dyn_dyn::DynDynTable::new(&#table_ident #turbo_tok #type_generics::__TABLE[..])
            }
        }
    };

    tokens
}
