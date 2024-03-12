use proc_macro::{Diagnostic, Level, Span};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{GenericParam, ItemImpl, Token, Type};

pub fn dyn_dyn_impl(args: Punctuated<Type, Token![,]>, input: ItemImpl) -> TokenStream {
    if input.trait_.is_none() {
        Diagnostic::spanned(
            proc_macro::Span::call_site(),
            Level::Error,
            "Cannot use dyn_dyn_impl on an inherent impl block",
        )
        .emit();
        return input.to_token_stream();
    } else if input.trait_.as_ref().unwrap().0.is_some() {
        Diagnostic::spanned(
            proc_macro::Span::call_site(),
            Level::Error,
            "Cannot use dyn_dyn_impl on a negative impl block",
        )
        .emit();
        return input.to_token_stream();
    }

    let self_ty = &input.self_ty;
    let trait_ = input.trait_.clone().unwrap().1;
    let generics = &input.generics;
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
            "dyn-dyn implementors cannot have lifetime arguments",
        )
        .emit();
        return input.to_token_stream();
    }

    let table_ident = format_ident!("__dyn_dyn_{}_DynTable", Span::call_site().line());
    let num_table_entries = args.len();

    let turbo_tok = if input.generics.params.is_empty() {
        None
    } else {
        Some(quote!(::))
    };

    let convert_tys = args.iter();

    let marker_contents = input.generics.params.iter().filter_map(|p| match *p {
        GenericParam::Type(ref p) => Some(p.ident.clone()),
        _ => None,
    });
    let marker_contents = quote!(#(#marker_contents),*);

    let tokens = quote! {
        #input

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        struct #table_ident #generics(#marker_contents) #where_clause;

        impl #impl_generics #table_ident #type_generics #where_clause {
            pub const __TABLE: [::dyn_dyn::DynDynTableEntry; #num_table_entries] = [
                #(
                    ::dyn_dyn::DynDynTableEntry::new::<#self_ty, dyn #convert_tys>()
                ),*
            ];
        }

        // SAFETY: The returned DynDynTable does not depend on data in self at all, so get_dyn_dyn_table will always return the same table
        //         as long as the metadata pointer is not changed in an unsafe way. All entries in the table have valid metadata for this
        //         type since they were retrieved by performing a trivial unsized coercion on a *const Self.
        unsafe impl #impl_generics ::dyn_dyn::internal::DynDynImpl<dyn #trait_> for #self_ty #where_clause {
            fn get_dyn_dyn_table(&self) -> ::dyn_dyn::DynDynTable {
                ::dyn_dyn::DynDynTable::new(&#table_ident #turbo_tok #type_generics::__TABLE[..])
            }
        }
    };

    tokens
}
