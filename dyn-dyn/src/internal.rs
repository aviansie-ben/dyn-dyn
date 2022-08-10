use crate::{DynDyn, DynDynFat, DynDynMut, DynDynTable, DynTrait};
use std::marker::{PhantomData, Unsize};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub unsafe trait DynDynDerived<B: ?Sized + DynDynBase> {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

pub trait DynDynBase {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

pub trait DerefHelperT<'a, 'b, B: ?Sized + DynDynBase, T: ?Sized + DynDyn<'b, B>> {
    fn __dyn_dyn_check_ref(self) -> Self;
    fn __dyn_dyn_check_fat(self) -> Self;
}

pub struct DerefHelper<'a, B: ?Sized + DynDynBase, T: ?Sized>(&'a T, PhantomData<fn(B) -> B>);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized> DerefHelper<'a, B, T> {
    pub fn new(val: &'a T) -> Self {
        Self(val, PhantomData)
    }
}

impl<'a, 'b, B: ?Sized + DynDynBase, T: ?Sized + DynDyn<'b, B>> DerefHelper<'a, B, T> {
    pub fn __dyn_dyn_deref(self) -> (&'a B, DynDynTable) {
        let (ptr, table) = DynDyn::<B>::deref_dyn_dyn(self.0);
        (ptr, table)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &'static T::DerefTarget {
        panic!("this method is only meant to be used for typechecking and should never be called")
    }
}

impl<'a, 'b, B: ?Sized + DynDynBase, T: ?Sized + DynDyn<'b, B>> DerefHelperT<'a, 'b, B, T>
    for DerefHelper<'a, B, T>
{
    fn __dyn_dyn_check_ref(self) -> Self {
        self
    }

    fn __dyn_dyn_check_fat(self) -> Self {
        self
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DerefHelper<'a, B, T> {
    pub fn __dyn_dyn_check_ref(self) -> DerefHelperRef<'a, B, T> {
        DerefHelperRef(self.0, self.1)
    }
}

impl<'a, B: ?Sized + DynDynBase, P: Deref> DerefHelper<'a, B, DynDynFat<B, P>>
where
    P::Target: Unsize<B>,
{
    pub fn __dyn_dyn_check_fat(self) -> DerefHelperFat<'a, B, P> {
        DerefHelperFat(self.0, self.1)
    }
}

pub struct DerefHelperRef<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>>(
    &'a T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DerefHelperRef<'a, B, T> {
    pub fn __dyn_dyn_check_fat(self) -> Self {
        self
    }

    pub fn __dyn_dyn_deref(self) -> (&'a B, DynDynTable) {
        let (_, table) = DynDyn::<B>::deref_dyn_dyn(&self.0);
        (self.0, table)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &'static T {
        panic!("this method is only meant to be used for typechecking and should never be called")
    }
}

pub struct DerefHelperFat<'a, B: ?Sized + DynDynBase, P: Deref>(
    &'a DynDynFat<B, P>,
    PhantomData<fn(B) -> B>,
)
where
    P::Target: Unsize<B>;

impl<'a, B: ?Sized + DynDynBase, P: Deref> DerefHelperFat<'a, B, P>
where
    P::Target: Unsize<B>,
{
    pub fn __dyn_dyn_deref(self) -> (&'a B, DynDynTable) {
        let table = DynDynFat::get_dyn_dyn_table(self.0);
        (&**self.0, table)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &'static P::Target {
        panic!("this method is only meant to be used for typechecking and should never be called")
    }
}

#[inline(always)]
pub unsafe fn try_downcast<B: ?Sized + DynDynBase, D: ?Sized + DynTrait, R: ?Sized>(
    (ptr, table): (&B, DynDynTable),
    f: impl Fn(*mut D) -> *mut R,
) -> Option<&R> {
    table
        .find_dyn(NonNull::from(ptr).cast())
        .map(|p| &*f(p.as_ptr()))
}

pub trait DerefMutHelperT<'a, 'b, B: ?Sized + DynDynBase, T: ?Sized + DynDyn<'b, B>> {
    fn __dyn_dyn_check_ref(self) -> Self;
    fn __dyn_dyn_check_fat(self) -> Self;
}

pub struct DerefMutHelper<'a, B: ?Sized + DynDynBase, T: ?Sized>(
    &'a mut T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized> DerefMutHelper<'a, B, T> {
    pub fn new(val: &'a mut T) -> Self {
        Self(val, PhantomData)
    }
}

impl<'a, 'b, B: ?Sized + DynDynBase, T: ?Sized + DynDynMut<'b, B>> DerefMutHelper<'a, B, T> {
    pub fn __dyn_dyn_deref(self) -> (&'a mut B, DynDynTable) {
        let (ptr, table) = DynDynMut::<B>::deref_mut_dyn_dyn(self.0);
        (ptr, table)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &'static T::DerefTarget {
        panic!("this method is only meant to be used for typechecking and should never be called")
    }
}

impl<'a, 'b, B: ?Sized + DynDynBase, T: ?Sized + DynDyn<'b, B>> DerefMutHelperT<'a, 'b, B, T>
    for DerefMutHelper<'a, B, T>
{
    fn __dyn_dyn_check_ref(self) -> Self {
        self
    }

    fn __dyn_dyn_check_fat(self) -> Self {
        self
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DerefMutHelper<'a, B, T> {
    pub fn __dyn_dyn_check_ref(self) -> DerefMutHelperRef<'a, B, T> {
        DerefMutHelperRef(self.0, self.1)
    }
}

impl<'a, B: ?Sized + DynDynBase, P: Deref> DerefMutHelper<'a, B, DynDynFat<B, P>>
where
    P::Target: Unsize<B>,
{
    pub fn __dyn_dyn_check_fat(self) -> DerefMutHelperFat<'a, B, P> {
        DerefMutHelperFat(self.0, self.1)
    }
}

pub struct DerefMutHelperRef<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>>(
    &'a mut T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DerefMutHelperRef<'a, B, T> {
    pub fn __dyn_dyn_check_fat(self) -> Self {
        self
    }

    pub fn __dyn_dyn_deref(mut self) -> (&'a mut B, DynDynTable) {
        let (_, table) = DynDynMut::<B>::deref_mut_dyn_dyn(&mut self.0);
        (self.0, table)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &'static T {
        panic!("this method is only meant to be used for typechecking and should never be called")
    }
}

pub struct DerefMutHelperFat<'a, B: ?Sized + DynDynBase, P: Deref>(
    &'a mut DynDynFat<B, P>,
    PhantomData<fn(B) -> B>,
)
where
    P::Target: Unsize<B>;

impl<'a, B: ?Sized + DynDynBase, P: DerefMut> DerefMutHelperFat<'a, B, P>
where
    P::Target: Unsize<B>,
{
    pub fn __dyn_dyn_deref(self) -> (&'a mut B, DynDynTable) {
        let table = DynDynFat::get_dyn_dyn_table(self.0);
        (&mut **self.0, table)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &'static P::Target {
        panic!("this method is only meant to be used for typechecking and should never be called")
    }
}

#[inline(always)]
pub unsafe fn try_downcast_mut<B: ?Sized + DynDynBase, D: ?Sized + DynTrait, R: ?Sized>(
    (ptr, table): (&mut B, DynDynTable),
    f: impl Fn(*mut D) -> *mut R,
) -> Option<&mut R> {
    table
        .find_dyn(NonNull::from(ptr).cast())
        .map(|p| &mut *f(p.as_ptr()))
}
