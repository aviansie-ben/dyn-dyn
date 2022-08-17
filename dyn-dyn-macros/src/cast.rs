use proc_macro::{Diagnostic, Level};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Expr, Token, TraitBound, TypeParamBound};

#[derive(Copy, Clone)]
pub enum DynDynCastType {
    Mut(Token![mut]),
    Move(Token![move]),
    Ref,
}

pub struct DynDynCastInput {
    ty: DynDynCastType,
    base_traits: Punctuated<TypeParamBound, Token![+]>,
    _arrow: Token![=>],
    target_traits: Punctuated<TypeParamBound, Token![+]>,
    _comma: Token![,],
    expr: Expr,
}

impl Parse for DynDynCastInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(DynDynCastInput {
            ty: if let Some(tok) = input.parse::<Option<Token![mut]>>()? {
                DynDynCastType::Mut(tok)
            } else if let Some(tok) = input.parse::<Option<Token![move]>>()? {
                DynDynCastType::Move(tok)
            } else {
                DynDynCastType::Ref
            },
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
    ty: DynDynCastType,
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
        ty: input.ty,
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
                ty,
                base_primary_trait,
                base_markers,
                tgt_primary_trait,
                tgt_markers,
            } = input_parsed;

            let helper_new = match ty {
                DynDynCastType::Mut(_) => quote!(new_mut),
                DynDynCastType::Move(_) => quote!(new_move),
                DynDynCastType::Ref => quote!(new_ref),
            };

            let check_markers = if !tgt_markers.is_empty() || !base_markers.is_empty() {
                quote!(
                    if false {
                        fn __dyn_dyn_marker_check(
                            r: &(impl ?Sized + #base_primary_trait #(+ #base_markers)*)
                        ) -> &(impl ?Sized + #base_primary_trait #(+ #tgt_markers)*) { r }

                        __dyn_dyn_marker_check(::dyn_dyn::internal::DerefHelperEnd::typecheck(&__dyn_dyn_input));
                    }
                )
            } else {
                quote!()
            };

            let cast_metadata = if !tgt_markers.is_empty() {
                quote! {
                    let __dyn_dyn_metadata = ::dyn_dyn::internal::cast_metadata::<
                        dyn #tgt_primary_trait, dyn #tgt_primary_trait #(+ #tgt_markers)*
                    >(__dyn_dyn_metadata, |__dyn_dyn_ptr| __dyn_dyn_ptr as *mut (dyn #tgt_primary_trait #(+ #tgt_markers)*));
                }
            } else {
                quote!()
            };

            quote!((|__dyn_dyn_input| unsafe {
                #check_markers

                let __dyn_dyn_table = ::dyn_dyn::internal::DerefHelperEnd::<dyn #base_primary_trait>::get_dyn_dyn_table(&__dyn_dyn_input);
                if true {
                    __dyn_dyn_table.find::<dyn #tgt_primary_trait>().map(|__dyn_dyn_metadata| {
                        #cast_metadata
                        ::dyn_dyn::internal::DerefHelperEnd::<dyn #base_primary_trait>::downcast_unchecked::<
                            dyn #tgt_primary_trait #(+ #tgt_markers)*
                        >(__dyn_dyn_input, __dyn_dyn_metadata)
                    })
                } else {
                    fn __dyn_dyn_constrain_lifetime<
                        '__dyn_dyn_ref,
                        '__dyn_dyn_life,
                        T: ::dyn_dyn::DynDyn<'__dyn_dyn_ref, dyn #base_primary_trait + '__dyn_dyn_life>
                    >(
                        _: T
                    ) -> <
                        T as ::dyn_dyn::DowncastUnchecked<'__dyn_dyn_ref, dyn #base_primary_trait + '__dyn_dyn_life>
                    >::DowncastResult<dyn #tgt_primary_trait #(+ #tgt_markers)* + '__dyn_dyn_life> {
                        unreachable!()
                    }

                    Some(__dyn_dyn_constrain_lifetime(
                        ::dyn_dyn::internal::DerefHelperEnd::<dyn #base_primary_trait>::unwrap(__dyn_dyn_input)
                    ))
                }
            })({
                use ::dyn_dyn::internal::DerefHelperT;

                ::dyn_dyn::internal::DerefHelper::<dyn #base_primary_trait, _>::#helper_new(#val)
                    .__dyn_dyn_check_dyn_dyn()
                    .__dyn_dyn_check_ref_mut_dyn_dyn()
                    .__dyn_dyn_check_ref_dyn_dyn()
                    .__dyn_dyn_check_deref_mut()
                    .__dyn_dyn_check_deref()
            }))
        }
        Err((span, err)) => {
            let err = match err {
                Error::FirstBoundMustBePrimaryTrait => "First bound must be the primary trait",
            };

            Diagnostic::spanned(span.unwrap(), Level::Error, err).emit();

            quote!(unreachable!())
        }
    }
}
