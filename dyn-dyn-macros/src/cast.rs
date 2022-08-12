use proc_macro::{Diagnostic, Level};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Expr, Token, TraitBound, TypeParamBound};

pub struct DynDynCastInput {
    mutability: Option<Token![mut]>,
    base_traits: Punctuated<TypeParamBound, Token![+]>,
    _arrow: Token![=>],
    target_traits: Punctuated<TypeParamBound, Token![+]>,
    _comma: Token![,],
    expr: Expr,
}

impl Parse for DynDynCastInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DynDynCastInput {
            mutability: input.parse()?,
            base_traits: Punctuated::parse_separated_nonempty(input)?,
            _arrow: input.parse()?,
            target_traits: Punctuated::parse_separated_nonempty(input)?,
            _comma: input.parse()?,
            expr: input.parse()?,
        })
    }
}

struct DynDynCastProcessedInput {
    val: Expr,
    is_mut: bool,
    base_primary_trait: TraitBound,
    base_markers: Vec<TypeParamBound>,
    tgt_primary_trait: TraitBound,
    tgt_markers: Vec<TypeParamBound>,
}

#[derive(Debug, Clone, Copy)]
enum Error {
    FirstBoundMustBePrimaryTrait,
}

fn split_trait_bounds(
    input: &Punctuated<TypeParamBound, Token![+]>,
) -> Result<(TraitBound, Vec<TypeParamBound>), (Span, Error)> {
    let primary_trait = match input[0] {
        TypeParamBound::Trait(ref bound) => bound.clone(),
        TypeParamBound::Lifetime(_) => {
            return Err((input[0].span(), Error::FirstBoundMustBePrimaryTrait));
        }
    };

    Ok((primary_trait, input.iter().skip(1).cloned().collect()))
}

fn process_input(input: &DynDynCastInput) -> Result<DynDynCastProcessedInput, (Span, Error)> {
    let (base_primary_trait, base_markers) = split_trait_bounds(&input.base_traits)?;
    let (tgt_primary_trait, tgt_markers) = split_trait_bounds(&input.target_traits)?;

    Ok(DynDynCastProcessedInput {
        val: input.expr.clone(),
        is_mut: input.mutability.is_some(),
        base_primary_trait,
        base_markers,
        tgt_primary_trait,
        tgt_markers,
    })
}

pub fn dyn_dyn_cast(input: DynDynCastInput) -> TokenStream {
    let input_parsed = process_input(&input);

    match input_parsed {
        Ok(input_parsed) => {
            let DynDynCastProcessedInput {
                val,
                is_mut,
                base_primary_trait,
                base_markers,
                tgt_primary_trait,
                tgt_markers,
            } = input_parsed;

            let (mut_tok, try_downcast, deref_helper) = if is_mut {
                (
                    quote!(mut),
                    quote!(try_downcast_mut),
                    quote!(DerefMutHelper),
                )
            } else {
                (quote!(), quote!(try_downcast), quote!(DerefHelper))
            };

            let check_markers = if !tgt_markers.is_empty() || !base_markers.is_empty() {
                quote!(
                    if false {
                        fn __dyn_dyn_marker_check(
                            r: &(impl ?Sized + #base_primary_trait #(+ #base_markers)*)
                        ) -> &(impl ?Sized + #base_primary_trait #(+ #tgt_markers)*) { r }

                        __dyn_dyn_marker_check(__dyn_dyn_input.__dyn_dyn_deref_typecheck());
                    }
                )
            } else {
                quote!()
            };

            quote!((|__dyn_dyn_input| unsafe {
                if true {
                    ::dyn_dyn::internal::#try_downcast::<dyn #base_primary_trait, dyn #tgt_primary_trait, _>(__dyn_dyn_input, |p| p as *mut (dyn #tgt_primary_trait #(+ #tgt_markers)*))
                } else {
                    fn __dyn_dyn_constrain_lifetime<'__dyn_dyn_ref, '__dyn_dyn_life>(
                        _: &'__dyn_dyn_ref #mut_tok (dyn #base_primary_trait + '__dyn_dyn_life)
                    ) -> &'__dyn_dyn_ref #mut_tok (dyn #tgt_primary_trait #(+ #tgt_markers)* + '__dyn_dyn_life) {
                        unreachable!()
                    }

                    Some(__dyn_dyn_constrain_lifetime(__dyn_dyn_input.0))
                }
            })({
                let __dyn_dyn_input = ::dyn_dyn::internal::#deref_helper::<dyn #base_primary_trait, _>::new(#val);

                {
                    use ::dyn_dyn::internal::DerefHelperT;

                    let __dyn_dyn_input = __dyn_dyn_input
                        .__dyn_dyn_check_ref()
                        .__dyn_dyn_check_trait()
                        .__dyn_dyn_check_deref();

                    #check_markers

                    __dyn_dyn_input.__dyn_dyn_deref()
                }
            }))
        }
        Err((span, err)) => {
            let err = match err {
                Error::FirstBoundMustBePrimaryTrait => "First bound must be the primary trait",
            };

            Diagnostic::spanned(span.unwrap(), Level::Error, err).emit();

            let DynDynCastInput {
                mutability,
                target_traits,
                ..
            } = input;
            quote! { (None as ::core::option::Option<&#mutability (dyn #target_traits)>) }
        }
    }
}
