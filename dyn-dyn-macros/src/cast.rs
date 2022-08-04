use crate::util::dyn_dyn_crate;
use proc_macro::{Diagnostic, Level};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    Expr, ExprCast, Lifetime, TraitBound, Type, TypeParamBound, TypeReference, TypeTraitObject,
};

struct DynDynCastInput {
    val: Expr,
    is_mut: bool,
    primary_trait: TraitBound,
    marker_traits: Vec<TraitBound>,
}

#[derive(Debug, Clone, Copy)]
enum Error {
    CastToNonTraitObject,
    NonStaticLifetime,
}

fn is_static_lifetime(lifetime: &Lifetime) -> bool {
    lifetime.ident.to_string() == "static"
}

fn parse_input(input: &ExprCast) -> Result<DynDynCastInput, (Span, Error)> {
    let (mutability, mut elem) = match *input.ty {
        Type::Reference(TypeReference {
            mutability,
            ref elem,
            ..
        }) => (mutability, &**elem),
        _ => {
            return Err((input.ty.span(), Error::CastToNonTraitObject));
        }
    };

    while let Type::Paren(ref inner) = *elem {
        elem = &*inner.elem;
    }

    let bounds = match *elem {
        Type::TraitObject(TypeTraitObject { ref bounds, .. }) => bounds,
        _ => {
            return Err((input.ty.span(), Error::CastToNonTraitObject));
        }
    };

    let bounds: Result<Vec<_>, _> = bounds
        .pairs()
        .filter_map(|pair| {
            let bound = pair.value();

            match **bound {
                TypeParamBound::Trait(ref bound) => Some(Ok(bound)),
                TypeParamBound::Lifetime(ref bound) => {
                    if is_static_lifetime(bound) {
                        None
                    } else {
                        Some(Err((bound.span(), Error::NonStaticLifetime)))
                    }
                }
            }
        })
        .collect();
    let bounds = bounds?;

    // Remove parentheses around the input expression to avoid falsely triggering the unused_parens lint
    let val = match *input.expr {
        Expr::Paren(ref val) => (*val.expr).clone(),
        ref val => val.clone(),
    };

    Ok(DynDynCastInput {
        val,
        is_mut: mutability.is_some(),
        primary_trait: bounds[0].clone(),
        marker_traits: bounds[1..].iter().map(|b| (*b).clone()).collect(),
    })
}

pub fn dyn_dyn_cast(input: ExprCast) -> TokenStream {
    let dyn_dyn = dyn_dyn_crate();
    let input_parsed = parse_input(&input);

    match input_parsed {
        Ok(input_parsed) => {
            let DynDynCastInput {
                val,
                is_mut,
                primary_trait,
                marker_traits,
            } = input_parsed;

            let (mut_tok, downcast_method) = if is_mut {
                (quote!(mut), quote!(try_downcast_mut))
            } else {
                (quote!(), quote!(try_downcast))
            };

            let check_markers = if let &[ref first_marker, ref other_markers @ ..] =
                &marker_traits[..]
            {
                quote!({
                    fn __dyn_dyn_marker_check(_: &(impl #first_marker #(+ #other_markers)* + ?Sized)) {}
                    __dyn_dyn_marker_check(__dyn_dyn_input);
                })
            } else {
                quote!()
            };

            quote!({
                let __dyn_dyn_input = #val;

                #check_markers

                ::core::option::Option::map(
                    #dyn_dyn::DynDyn::#downcast_method::<dyn #primary_trait>(__dyn_dyn_input),
                    |__dyn_dyn_result| unsafe {
                        & #mut_tok *(
                            __dyn_dyn_result
                                as *const dyn #primary_trait
                                as *mut dyn #primary_trait
                                as *mut (dyn #primary_trait #(+ #marker_traits)*)
                        )
                    }
                )
            })
        }
        Err((span, err)) => {
            let err = match err {
                Error::CastToNonTraitObject => {
                    "Dyn-dyn cast should target a reference to a trait object"
                }
                Error::NonStaticLifetime => {
                    "Dyn-dyn cast bounds cannot include non-static lifetimes"
                }
            };

            Diagnostic::spanned(span.unwrap(), Level::Error, err).emit();

            let ty = &input.ty;
            quote! { (None as Option<#ty>) }
        }
    }
}
