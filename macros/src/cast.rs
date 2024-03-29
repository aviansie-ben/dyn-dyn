use proc_macro::{Diagnostic, Level};
use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{quote, TokenStreamExt};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Bracket;
use syn::{bracketed, Expr, Token, TraitBound, Type, TypeParamBound};

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
    outer_struct: Option<(Bracket, TokenStream)>,
    _comma: Token![,],
    expr: Expr,
}

impl Parse for DynDynCastInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let outer_struct;

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
            outer_struct: if input.peek(Bracket) {
                Some((bracketed!(outer_struct in input), outer_struct.parse()?))
            } else {
                None
            },
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
    outer_struct: Option<TokenStream>,
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
        outer_struct: input
            .outer_struct
            .as_ref()
            .map(|&(_, ref outer_struct)| outer_struct.clone()),
    })
}

fn replace_placeholder(input: Option<TokenStream>, val: TokenStream) -> Result<TokenStream, ()> {
    fn replace_placeholder_impl(
        input: TokenStream,
        val: &mut Option<TokenStream>,
    ) -> Result<TokenStream, ()> {
        let mut out = TokenStream::new();

        for tt in input {
            let tt = match tt {
                TokenTree::Group(group) => TokenTree::Group(Group::new(
                    group.delimiter(),
                    replace_placeholder_impl(group.stream(), val)?,
                )),
                TokenTree::Punct(ref p) if p.as_char() == '$' => {
                    if let Some(val) = val.take() {
                        TokenTree::Group(Group::new(Delimiter::None, val))
                    } else {
                        return Err(());
                    }
                }
                tt => tt,
            };

            out.append(tt);
        }

        Ok(out)
    }

    if let Some(input) = input {
        let mut val = Some(val);

        let out = replace_placeholder_impl(input, &mut val)?;

        if val.is_some() {
            Err(())
        } else {
            Ok(out)
        }
    } else {
        Ok(val)
    }
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
                outer_struct,
            } = input_parsed;

            let helper_new = match ty {
                DynDynCastType::Mut(_) => quote!(new_mut),
                DynDynCastType::Move(_) => quote!(new_move),
                DynDynCastType::Ref => quote!(new_ref),
            };

            if let Some(ref outer_struct) = outer_struct.as_ref() {
                match replace_placeholder(Some((*outer_struct).clone()), quote!(())) {
                    Ok(outer_struct) => match syn::parse2::<Type>(outer_struct) {
                        Ok(_) => {}
                        Err(err) => {
                            return err.to_compile_error();
                        }
                    },
                    Err(()) => {
                        Diagnostic::spanned(
                            outer_struct.span().unwrap(),
                            Level::Error,
                            "outer struct must have exactly one placeholder `$`",
                        )
                        .emit();

                        return quote!(unreachable!());
                    }
                }
            }

            let check_markers = if !tgt_markers.is_empty() || !base_markers.is_empty() {
                let impl_base_markers = replace_placeholder(
                    outer_struct.clone(),
                    quote!((impl ?Sized + #base_primary_trait #(+ #base_markers)*)),
                )
                .unwrap();

                let impl_tgt_markers = replace_placeholder(
                    outer_struct.clone(),
                    quote!((impl ?Sized + #base_primary_trait #(+ #tgt_markers)*)),
                )
                .unwrap();

                quote!(
                    if false {
                        fn __dyn_dyn_marker_check(
                            r: &#impl_base_markers
                        ) -> &#impl_tgt_markers { r }

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

            let primary_base =
                replace_placeholder(outer_struct.clone(), quote!((dyn #base_primary_trait)))
                    .unwrap();
            let tgt_dyn = replace_placeholder(
                outer_struct.clone(),
                quote!((dyn #tgt_primary_trait #(+ #tgt_markers)*)),
            )
            .unwrap();

            let constrain_lifetime = {
                let base_with_lifetime = replace_placeholder(
                    outer_struct.clone(),
                    quote!((dyn #base_primary_trait + '__dyn_dyn_life)),
                )
                .unwrap();
                let tgt_with_lifetime = replace_placeholder(
                    outer_struct,
                    quote!((dyn #tgt_primary_trait #(+ #tgt_markers)* + '__dyn_dyn_life)),
                )
                .unwrap();

                quote!({
                    fn __dyn_dyn_constrain_lifetime<
                        '__dyn_dyn_ref,
                        '__dyn_dyn_life,
                        T: ::dyn_dyn::internal::DynDynConstrainLifetime<'__dyn_dyn_ref, #base_with_lifetime>
                    >(
                        _: T
                    ) -> <
                        T as ::dyn_dyn::internal::DynDynConstrainLifetime<'__dyn_dyn_ref, #base_with_lifetime>
                    >::Result<#tgt_with_lifetime> {
                        unreachable!()
                    }

                    ::core::result::Result::Ok(__dyn_dyn_constrain_lifetime(
                        ::dyn_dyn::internal::DerefHelperEnd::<#primary_base>::unwrap(__dyn_dyn_input)
                    ))
                })
            };

            quote!((|__dyn_dyn_input| {
                #check_markers

                let __dyn_dyn_table = ::dyn_dyn::internal::DerefHelperEnd::<#primary_base>::get_dyn_dyn_table(&__dyn_dyn_input);
                if true {
                    if let ::core::option::Option::Some(__dyn_dyn_metadata) = __dyn_dyn_table.find::<dyn #tgt_primary_trait>() {
                        #cast_metadata

                        // SAFETY:
                        //
                        // By the safety invariants of GetDynDynTable<B>, we know that the returned DynDynTable matches the concrete type of
                        // the pointee, so attaching it to the pointer is valid. The cast performed by cast_metadata is also known to be
                        // valid, since check_markers will not compile unless the pointee type of the input implements all of the necessary
                        // marker traits and the actual metadata cast is performed via use of the "as" operation to cast it using a fake fat
                        // pointer.
                        //
                        // Additionally, the lifetime of the output is constrained by the result of the other side of this if
                        // statement, where __dyn_dyn_constrain_lifetime is called. By doing this, we ensure that the pointee of the
                        // output cannot outlive the pointee of the input, so there's no lifetime extension here.
                        unsafe {
                            ::core::result::Result::Ok(::dyn_dyn::internal::DerefHelperEnd::<#primary_base>::downcast_unchecked::<
                                #tgt_dyn
                            >(__dyn_dyn_input, __dyn_dyn_metadata))
                        }
                    } else {
                        ::core::result::Result::Err(::dyn_dyn::internal::DerefHelperEnd::<#primary_base>::into_err(
                            __dyn_dyn_input
                        ))
                    }
                } else {
                    #constrain_lifetime
                }
            })({
                use ::dyn_dyn::internal::DerefHelperT;

                ::dyn_dyn::internal::DerefHelper::<#primary_base, _>::#helper_new(#val)
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
