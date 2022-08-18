use crate::{
    DowncastUnchecked, DynDyn, DynDynCastTarget, DynDynRef, DynDynRefMut, DynDynTable,
    GetDynDynTable,
};
use core::marker::{PhantomData, Unsize};
use core::ops::{Deref, DerefMut};
use core::ptr::DynMetadata;
use core::ptr::NonNull;
use stable_deref_trait::StableDeref;

#[allow(clippy::missing_safety_doc)] // This module is marked doc(hidden)
pub unsafe trait DynDynDerived<B: ?Sized + DynDynBase> {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

#[allow(clippy::missing_safety_doc)] // This module is marked doc(hidden)
pub unsafe trait DynDynBase {
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
    unsafe fn downcast_unchecked<D: ?Sized + DynDynCastTarget>(
        self,
        metadata: DynMetadata<D::Root>,
    ) -> <Self::Inner as DowncastUnchecked<'a, B>>::DowncastResult<D>;
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

    unsafe fn downcast_unchecked<D: ?Sized + DynDynCastTarget>(
        self,
        metadata: DynMetadata<D::Root>,
    ) -> <Self::Inner as DowncastUnchecked<'a, B>>::DowncastResult<D> {
        self.0.downcast_unchecked(metadata)
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
