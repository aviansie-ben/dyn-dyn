//! This module contains internal implementation details that are meant to be used as part of the
//! implementation of the macros. They are marked as public to allow them to be used from the
//! expansions, but they are not actually part of the public API of this crate and should not be
//! directly relied upon by external code.

use crate::{
    DowncastUnchecked, DynDyn, DynDynBase, DynDynCastTarget, DynDynRef, DynDynRefMut, DynDynTable,
    GetDynDynTable,
};
use core::marker::{PhantomData, Unsize};
use core::ops::{Deref, DerefMut};
use core::ptr::{DynMetadata, NonNull, Pointee};
use stable_deref_trait::StableDeref;

// This special proxy trait is needed for the __dyn_dyn_constrain_lifetime method which is
// generated internally by the dyn_dyn_cast! macro to constrain the lifetime of its result. While
// it seems like we should be able to use DynDyn and DowncastUnchecked directly, this actually
// results in compilation errors.
//
// If (dyn Base + '__dyn_dyn_life) doesn't show up in the return type in a very particular way, the
// compiler will complain about '__dyn_dyn_life not outliving 'static when the base trait itself is
// marked as 'static. This proxy trait allows the return type to still include the base trait as
// part of writing it out to avoid this.
pub trait DynDynConstrainLifetime<'a, B: ?Sized + DynDynBase> {
    type Result<D: ?Sized + 'a>;
}

impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B>> DynDynConstrainLifetime<'a, B> for T {
    type Result<D: ?Sized + 'a> = <T as DowncastUnchecked<'a>>::DowncastResult<D>;
}

#[allow(clippy::missing_safety_doc)] // This module is marked doc(hidden)
pub unsafe trait DynDynImpl<B: ?Sized + DynDynBase> {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

pub trait DerefHelperT: Sized {
    fn __dyn_dyn_check_dyn_dyn(self) -> Self {
        self
    }

    fn __dyn_dyn_check_ref_mut_dyn_dyn(self) -> Self {
        self
    }

    fn __dyn_dyn_check_ref_dyn_dyn(self) -> Self {
        self
    }

    fn __dyn_dyn_check_deref_mut(self) -> Self {
        self
    }

    fn __dyn_dyn_check_deref(self) -> Self {
        self
    }
}

pub trait DerefHelperEnd<'a, B: ?Sized + DynDynBase> {
    type Inner: DynDyn<'a, B>;
    type Err;

    fn get_dyn_dyn_table(&self) -> DynDynTable;
    unsafe fn downcast_unchecked<D: ?Sized + Pointee>(
        self,
        metadata: <D as Pointee>::Metadata,
    ) -> <Self::Inner as DowncastUnchecked<'a>>::DowncastResult<D>;
    fn unwrap(self) -> Self::Inner;
    fn into_err(self) -> Self::Err;
    fn typecheck(&self) -> &<Self::Inner as GetDynDynTable<B>>::DynTarget {
        panic!("this method is only meant to be used for typechecking");
    }
}

pub struct DerefHelper<B: ?Sized + DynDynBase, T>(T, PhantomData<fn(B) -> B>);

impl<B: ?Sized + DynDynBase, T> DerefHelperT for DerefHelper<B, T> {}

impl<B: ?Sized + DynDynBase, T> DerefHelper<B, T> {
    pub fn new_move(val: T) -> Self {
        DerefHelper(val, PhantomData)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized> DerefHelper<B, &'a T> {
    pub fn new_ref(val: &'a T) -> Self {
        DerefHelper(val, PhantomData)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized> DerefHelper<B, &'a mut T> {
    pub fn new_mut(val: &'a mut T) -> Self {
        DerefHelper(val, PhantomData)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B>> DerefHelper<B, T> {
    pub fn __dyn_dyn_check_dyn_dyn(self) -> DerefHelperResolved<'a, B, T, T, impl FnOnce(T) -> T> {
        DerefHelperResolved(self.0, |x| x, self.1, PhantomData)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B> + StableDeref + DerefMut>
    DerefHelper<B, &'a mut T>
where
    T::Target: Unsize<B>,
{
    pub fn __dyn_dyn_check_ref_mut_dyn_dyn(
        self,
    ) -> DerefHelperResolved<
        'a,
        B,
        DynDynRefMut<'a, B, T>,
        &'a mut T::Target,
        impl FnOnce(DynDynRefMut<'a, B, T>) -> &'a mut T::Target,
    > {
        DerefHelperResolved(
            DynDynRefMut::new(self.0),
            |x| &mut **x.0,
            self.1,
            PhantomData,
        )
    }
}

impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B> + StableDeref> DerefHelper<B, &'a T>
where
    T::Target: Unsize<B>,
{
    pub fn __dyn_dyn_check_ref_dyn_dyn(
        self,
    ) -> DerefHelperResolved<
        'a,
        B,
        DynDynRef<'a, B, T>,
        &'a T::Target,
        impl FnOnce(DynDynRef<'a, B, T>) -> &'a T::Target,
    > {
        DerefHelperResolved(DynDynRef::new(self.0), |x| &**x.0, self.1, PhantomData)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: DerefMut> DerefHelper<B, &'a mut T>
where
    T::Target: Unsize<B>,
{
    pub fn __dyn_dyn_check_deref_mut(
        self,
    ) -> DerefHelperResolved<
        'a,
        B,
        &'a mut T::Target,
        &'a mut T::Target,
        impl FnOnce(&'a mut T::Target) -> &'a mut T::Target,
    > {
        DerefHelperResolved(&mut **self.0, |x| x, self.1, PhantomData)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: Deref> DerefHelper<B, &'a T>
where
    T::Target: Unsize<B>,
{
    pub fn __dyn_dyn_check_deref(
        self,
    ) -> DerefHelperResolved<
        'a,
        B,
        &'a T::Target,
        &'a T::Target,
        impl FnOnce(&'a T::Target) -> &'a T::Target,
    > {
        DerefHelperResolved(&**self.0, |x| x, self.1, PhantomData)
    }
}

// This impl should never actually be used, since we'll always switch over to DerefHelperResolved when calling __dyn_dyn_check_dyn_dyn. This
// impl is only actually here to get a friendlier error message if the value passed in does not match any of the checks. If we didn't have
// this impl, the compiler would complain that DerefHelper<B, T> doesn't implement DerefHelperEnd<'_, B>, but with this impl it instead
// complains that the input type doesn't implement the necessary traits for DynDyn<'_, B>.
impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B>> DerefHelperEnd<'a, B> for DerefHelper<B, T> {
    type Inner = T;
    type Err = T;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        unreachable!()
    }

    unsafe fn downcast_unchecked<D: ?Sized + Pointee>(
        self,
        _: <D as Pointee>::Metadata,
    ) -> <Self::Inner as DowncastUnchecked<'a>>::DowncastResult<D> {
        unreachable!()
    }

    fn unwrap(self) -> Self::Inner {
        unreachable!()
    }

    fn into_err(self) -> Self::Err {
        unreachable!()
    }
}

pub struct DerefHelperResolved<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B>, E, F: FnOnce(T) -> E>(
    T,
    F,
    PhantomData<fn(B) -> B>,
    PhantomData<&'a ()>,
);

impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B>, E, F: FnOnce(T) -> E> DerefHelperT
    for DerefHelperResolved<'a, B, T, E, F>
{
}

impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B>, E, F: FnOnce(T) -> E> DerefHelperEnd<'a, B>
    for DerefHelperResolved<'a, B, T, E, F>
{
    type Inner = T;
    type Err = E;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        self.0.get_dyn_dyn_table()
    }

    unsafe fn downcast_unchecked<D: ?Sized + Pointee>(
        self,
        metadata: <D as Pointee>::Metadata,
    ) -> <Self::Inner as DowncastUnchecked<'a>>::DowncastResult<D> {
        // SAFETY: Invariants are passed through
        unsafe { self.0.downcast_unchecked(metadata) }
    }

    fn unwrap(self) -> Self::Inner {
        self.0
    }

    fn into_err(self) -> Self::Err {
        self.1(self.0)
    }
}

pub fn cast_metadata<T: ?Sized + DynDynCastTarget, U: ?Sized + DynDynCastTarget>(
    meta: DynMetadata<T::Root>,
    f: impl Fn(*mut T) -> *mut U,
) -> DynMetadata<U::Root> {
    U::ptr_into_parts(
        NonNull::new(f(T::ptr_from_parts(NonNull::dangling(), meta).as_ptr())).unwrap(),
    )
    .1
}
