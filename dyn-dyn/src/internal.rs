use crate::{DynDyn, DynDynMut, DynDynTable, DynTrait};
use core::marker::{PhantomData, Unsize};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

#[allow(clippy::missing_safety_doc)] // This module is marked doc(hidden)
pub unsafe trait DynDynDerived<B: ?Sized + DynDynBase> {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

#[allow(clippy::missing_safety_doc)] // This module is marked doc(hidden)
pub unsafe trait DynDynBase {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

// In order to select the proper method of getting the base reference and table when using the dyn_dyn_cast! macro, we use some method
// resolution tricks to simulate specialization. These dummy trait methods will be invoked by the macro if the bounds on the corresponding
// impl block on DerefHelper aren't matched. This means that after calling all of them in succession, we end up with an object having a
// __dyn_dyn_deref() method that does the thing we want. The order of resolution we want is:
//
//   - Any reference &T where T implements B
//   - Any reference &T where T implements DynDyn<B>
//   - Any reference &T where T implements Deref which produces a reference to a type that implements B
pub trait DerefHelperT {
    fn __dyn_dyn_check_ref(self) -> Self;
    fn __dyn_dyn_check_trait(self) -> Self;
    fn __dyn_dyn_check_deref(self) -> Self;
}

pub struct DerefHelper<'a, B: ?Sized + DynDynBase, T: ?Sized>(&'a T, PhantomData<fn(B) -> B>);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized> DerefHelper<'a, B, T> {
    pub fn new(val: &'a T) -> Self {
        Self(val, PhantomData)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized> DerefHelperT for DerefHelper<'a, B, T> {
    fn __dyn_dyn_check_ref(self) -> Self {
        self
    }

    fn __dyn_dyn_check_trait(self) -> Self {
        self
    }

    fn __dyn_dyn_check_deref(self) -> Self {
        self
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DerefHelper<'a, B, T> {
    pub fn __dyn_dyn_check_ref(self) -> DerefHelperRef<'a, B, T> {
        DerefHelperRef(self.0, self.1)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + DynDyn<B>> DerefHelper<'a, B, T> {
    pub fn __dyn_dyn_check_trait(self) -> DerefHelperDynDyn<'a, B, T> {
        DerefHelperDynDyn(self.0, self.1)
    }

    // This method should never actually be called, since __dyn_dyn_check_trait should always result in a DerefHelperTrait with these trait
    // bounds. This method is only actually here so that we get a reasonable error message if all the checks fail.
    pub fn __dyn_dyn_deref(self) -> (&'a B, DynDynTable) {
        unreachable!()
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Deref> DerefHelper<'a, B, T>
where
    T::Target: Unsize<B>,
{
    pub fn __dyn_dyn_check_deref(self) -> DerefHelperRef<'a, B, T::Target> {
        DerefHelperRef(&**self.0, self.1)
    }
}

pub struct DerefHelperRef<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>>(
    &'a T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DerefHelperRef<'a, B, T> {
    pub fn __dyn_dyn_check_trait(self) -> Self {
        self
    }

    pub fn __dyn_dyn_check_deref(self) -> Self {
        self
    }

    pub fn __dyn_dyn_deref(self) -> (&'a B, DynDynTable) {
        let (_, table) = DynDyn::<B>::deref_dyn_dyn(&self.0);
        (self.0, table)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &T {
        panic!("this method is only meant to be used for typechecking and should never be called")
    }
}

pub struct DerefHelperDynDyn<'a, B: ?Sized + DynDynBase, T: ?Sized + DynDyn<B>>(
    &'a T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + DynDyn<B>> DerefHelperDynDyn<'a, B, T> {
    pub fn __dyn_dyn_check_deref(self) -> Self {
        self
    }

    pub fn __dyn_dyn_deref(self) -> (&'a B, DynDynTable) {
        DynDyn::<B>::deref_dyn_dyn(self.0)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &T::Target {
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

pub struct DerefMutHelper<'a, B: ?Sized + DynDynBase, T: ?Sized>(
    &'a mut T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized> DerefMutHelper<'a, B, T> {
    pub fn new(val: &'a mut T) -> Self {
        Self(val, PhantomData)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized> DerefHelperT for DerefMutHelper<'a, B, T> {
    fn __dyn_dyn_check_ref(self) -> Self {
        self
    }

    fn __dyn_dyn_check_trait(self) -> Self {
        self
    }

    fn __dyn_dyn_check_deref(self) -> Self {
        self
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DerefMutHelper<'a, B, T> {
    pub fn __dyn_dyn_check_ref(self) -> DerefMutHelperRef<'a, B, T> {
        DerefMutHelperRef(self.0, self.1)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + DynDynMut<B>> DerefMutHelper<'a, B, T> {
    pub fn __dyn_dyn_check_trait(self) -> DerefMutHelperDynDyn<'a, B, T> {
        DerefMutHelperDynDyn(self.0, self.1)
    }

    // This method should never actually be called, since __dyn_dyn_check_trait should always result in a DerefHelperTrait with these trait
    // bounds. This method is only actually here so that we get a reasonable error message if all the checks fail.
    pub fn __dyn_dyn_deref(self) -> (&'a mut B, DynDynTable) {
        unreachable!()
    }
}

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + DerefMut> DerefMutHelper<'a, B, T>
where
    T::Target: Unsize<B>,
{
    pub fn __dyn_dyn_check_deref(self) -> DerefMutHelperRef<'a, B, T::Target> {
        DerefMutHelperRef(&mut **self.0, self.1)
    }
}

pub struct DerefMutHelperRef<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>>(
    &'a mut T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DerefMutHelperRef<'a, B, T> {
    pub fn __dyn_dyn_check_trait(self) -> Self {
        self
    }

    pub fn __dyn_dyn_check_deref(self) -> Self {
        self
    }

    pub fn __dyn_dyn_deref(mut self) -> (&'a mut B, DynDynTable) {
        let (_, table) = DynDynMut::<B>::deref_mut_dyn_dyn(&mut self.0);
        (self.0, table)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &T {
        panic!("this method is only meant to be used for typechecking and should never be called")
    }
}

pub struct DerefMutHelperDynDyn<'a, B: ?Sized + DynDynBase, T: ?Sized + DynDynMut<B>>(
    &'a mut T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: ?Sized + DynDynMut<B>> DerefMutHelperDynDyn<'a, B, T> {
    pub fn __dyn_dyn_check_deref(self) -> Self {
        self
    }

    pub fn __dyn_dyn_deref(self) -> (&'a mut B, DynDynTable) {
        DynDynMut::<B>::deref_mut_dyn_dyn(self.0)
    }

    pub fn __dyn_dyn_deref_typecheck(&self) -> &T::Target {
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
