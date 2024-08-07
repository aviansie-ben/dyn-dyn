#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::needless_borrowed_reference)]
#![forbid(unsafe_op_in_unsafe_fn)]
#![feature(coerce_unsized)]
#![feature(const_type_id)]
#![cfg_attr(feature = "dynamic-names", feature(const_type_name))]
#![feature(doc_auto_cfg)]
#![feature(ptr_metadata)]
#![feature(unsize)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod cast_target;
mod fat;
mod table;

#[doc(hidden)]
pub mod internal;

/// Declares a trait as being a base trait for downcasting.
///
/// This macro marks a trait as being a base for dynamic trait object downcasting. All `impl` blocks for this trait will need to use the
/// [`#[dyn_dyn_impl]`](dyn_dyn_impl) attribute to declare what traits they wish to expose.
pub use dyn_dyn_macros::dyn_dyn_base;

/// Performs a dynamic downcast of a reference to a trait object where the trait was declared with [`#[dyn_dyn_base]`](dyn_dyn_base).
///
/// This macro allows for trying to cast such a reference to a reference to another trait object, returning an [`Option`] containing the
/// reference to the downcast trait object if the object in question implements that trait.
///
/// This macro accepts the following types for a given base trait `B`, with the first matching set of conditions determining how the
/// dereference will occur:
///
/// - A (mutable) reference to a type that implements `B`, returning a (mutable) reference referring to the same object as the original
///   reference
/// - A (mutable) reference to a pointer type that implements [`DynDyn<B>`], returning a (mutable) reference referring to the pointee of
///   that pointer
/// - A (mutable) reference to a pointer type that implements Deref with a target that implements `B`, returning a (mutable) reference
///   referring to the pointee of that pointer
///
/// # Examples
///
/// ```rust
/// # use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_impl};
/// #[dyn_dyn_base]
/// trait Base {}
/// trait Trait {}
///
/// struct Struct;
///
/// #[dyn_dyn_impl(Trait)]
/// impl Base for Struct {}
/// impl Trait for Struct {}
///
/// fn downcast(r: &dyn Base) -> Result<&dyn Trait, &dyn Base> {
///     dyn_dyn_cast!(Base => Trait, r)
/// }
///
/// fn downcast_mut(r: &mut dyn Base) -> Result<&mut dyn Trait, &mut dyn Base> {
///     dyn_dyn_cast!(mut Base => Trait, r)
/// }
///
/// # #[cfg(feature = "alloc")]
/// fn downcast_box(r: Box<dyn Base>) -> Result<Box<dyn Trait>, Box<dyn Base>> {
///     dyn_dyn_cast!(move Base => Trait, r)
/// }
///
/// let mut s = Struct;
///
/// assert!(downcast(&s).is_ok());
/// assert!(downcast_mut(&mut s).is_ok());
/// # #[cfg(feature = "alloc")]
/// assert!(downcast_box(Box::new(s)).is_ok());
/// ```
pub use dyn_dyn_macros::dyn_dyn_cast;

/// Marks an `impl` block as targeting a trait that was declared with the [`#[dyn_dyn_base]`](dyn_dyn_base) attribute.
///
/// This attribute allows the `impl` block to specify what other traits should be exposed for downcasting via the base trait that's being
/// implemented in this block.
///
/// # Examples
///
/// ```rust
/// # use core::fmt::Debug;
/// # use dyn_dyn::{dyn_dyn_base, dyn_dyn_impl};
/// #[dyn_dyn_base]
/// trait Base {}
///
/// #[derive(Debug)]
/// struct Struct;
///
/// #[dyn_dyn_impl(Debug)]
/// impl Base for Struct {}
/// ```
///
/// ```rust
/// # use dyn_dyn::{dyn_dyn_base, dyn_dyn_impl};
/// #[dyn_dyn_base]
/// trait Base {}
/// trait Trait<T> {}
///
/// struct Struct<T>(T);
///
/// impl<T> Trait<T> for Struct<T> {}
///
/// #[dyn_dyn_impl(Trait<T>)]
/// impl<T: 'static> Base for Struct<T> {}
/// ```
pub use dyn_dyn_macros::dyn_dyn_impl;

pub use cast_target::DynDynCastTarget;
pub use fat::DynDynFat;
pub use table::{AnyDynMetadata, DynDynTable, DynDynTableEntry, DynDynTableIterator};

#[cfg(doc)]
use core::ops::Deref;

use cfg_if::cfg_if;
use core::marker::{PhantomData, Unsize};
use core::ops::DerefMut;
use core::ptr::{self, Pointee};
use stable_deref_trait::StableDeref;

/// A type that can be dynamically downcast to other traits using the [`dyn_dyn_cast!`] macro.
///
/// This trait should not be manually implemented by user code. Instead, this trait should be implemented by using the
/// [`#[dyn_dyn_base]`](dyn_dyn_base) attribute on the trait in question. The exact shape of this trait is subject to change at any time, so
/// it generally shouldn't be relied upon in external code except as a trait bound.
///
/// # Safety
///
/// The result of calling [`DynDynBase::get_dyn_dyn_table`] on an object through a given base must never change for the lifetime of that
/// object, even if the object itself is mutated.
pub unsafe trait DynDynBase {
    /// Gets the [`DynDynTable`] for this object, for traits exposed via this base trait.
    ///
    /// In user code, it is generally preferred to use the implementation of [`GetDynDynTable`] for references rather than calling this
    /// method directly to avoid potential future breakage.
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

/// Wraps a reference to a pointer implementing [`GetDynDynTable<B>`] and which can be dereferenced to perform the downcast.
///
/// Using [`dyn_dyn_cast!`] on this struct will call [`GetDynDynTable::get_dyn_dyn_table`] on the pointer itself, then dereference this
/// pointer to perform the downcast. This allows a pointer implementing [`DynDyn<B>`] to be downcast into a reference without moving the
/// pointer itself.
pub struct DynDynRef<'a, B: ?Sized + DynDynBase, T: GetDynDynTable<B> + StableDeref>(
    &'a T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: GetDynDynTable<B> + StableDeref> DynDynRef<'a, B, T>
where
    T::Target: Unsize<B>,
{
    /// Creates a new [`DynDynRef`] for the provided reference to a pointer.
    pub fn new(r: &'a T) -> Self {
        DynDynRef(r, PhantomData)
    }
}

/// Wraps a mutable reference to a pointer implementing [`GetDynDynTable<B>`] and which can be dereferenced to perform the downcast.
///
/// Using [`dyn_dyn_cast!`] on this struct will call [`GetDynDynTable::get_dyn_dyn_table`] on the pointer itself, then dereference this
/// pointer to perform the downcast. This allows a pointer implementing [`DynDyn<B>`] to be downcast into a mutable reference without moving
/// the pointer itself.
pub struct DynDynRefMut<'a, B: ?Sized + DynDynBase, T: GetDynDynTable<B> + StableDeref + DerefMut>(
    &'a mut T,
    PhantomData<fn(B) -> B>,
);

impl<'a, B: ?Sized + DynDynBase, T: GetDynDynTable<B> + StableDeref + DerefMut>
    DynDynRefMut<'a, B, T>
{
    /// Creates a new [`DynDynRefMut`] for the provided mutable reference to a pointer.
    pub fn new(r: &'a mut T) -> Self {
        DynDynRefMut(r, PhantomData)
    }
}

/// A pointer to an object which has a [`DynDynTable`] associated with it.
///
/// # Safety
///
/// - If this type implements [`Deref`], then the reference returned by calling [`Deref::deref`] must not change for the lifetime of this
///   pointer unless the pointer itself is mutated.
/// - If this type implements [`DerefMut`], then the reference returned by calling [`DerefMut::deref_mut`] must not change for the lifetime
///   of this pointer unless the pointer itself is mutated and must point to the same object as a reference returned by calling
///   [`Deref::deref`], including having identical metadata. Additionally, calling [`DerefMut::deref_mut`] must not mutate the pointer.
/// - If this type implements [`Deref`], then the reference returned by calling [`Deref::deref`] must be unsize-coercible to a reference to
///   [`GetDynDynTable::get_dyn_dyn_table`].
/// - If this type implements [`Deref`], then the returned table must be equivalent to calling [`GetDynDynTable::get_dyn_dyn_table`] on a
///   reference returned by calling [`Deref::deref`].
/// - If this type implements [`DowncastUnchecked`], then the result of calling [`DowncastUnchecked::downcast_unchecked`] with
///   metadata retrieved from the table returned by calling [`GetDynDynTable::get_dyn_dyn_table`] on this pointer shall be valid and safe to
///   use.
pub unsafe trait GetDynDynTable<B: ?Sized + DynDynBase> {
    /// The actual type that this pointer currently points to. This type is used to allow propagation of auto trait bounds such as `Send`
    /// and `Sync` in the `dyn_dyn_cast!` macro.
    type DynTarget: ?Sized + Unsize<B>;

    /// Gets the [`DynDynTable`] for the object that this pointer points to.
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

/// A pointer to an object that can be unsafely downcast to point to another type.
pub trait DowncastUnchecked<'a> {
    /// The result of downcasting this pointer to point to the type `D`. Note that this type need not have the same outer wrapper as the
    /// type implementing `DowncastUnchecked`, since the result of the downcast may involve coercions and dereferences.
    type DowncastResult<D: ?Sized + 'a>;

    /// Downcasts this pointer into a new pointer pointing to the same object, but having type `D`.
    ///
    /// Generally, the result of calling this function should be equivalent to turning this pointer type into a raw pointer, removing its
    /// metadata, unsafely casting that pointer into a pointer to `D` using the provided metadata, and then turning that raw pointer into
    /// another pointer type.
    ///
    /// As long as the concrete type of the pointee matches the concrete type of the metadata provided, then this is guaranteed to result
    /// in a pointer which is valid and safe to use.
    ///
    /// # Safety
    ///
    /// Attaching the provided metadata to a pointer to the same data address as that held by this pointer must be guaranteed to be valid
    /// and safe to use before this function can be called.
    unsafe fn downcast_unchecked<D: ?Sized + Pointee>(
        self,
        metadata: <D as Pointee>::Metadata,
    ) -> Self::DowncastResult<D>;
}

/// A pointer object that can be safely downcast to refer to other trait types by using the `dyn_dyn_cast!` macro.
pub trait DynDyn<'a, B: ?Sized + DynDynBase>: GetDynDynTable<B> + DowncastUnchecked<'a> {}

impl<'a, B: ?Sized + DynDynBase, T: GetDynDynTable<B> + DowncastUnchecked<'a>> DynDyn<'a, B> for T {}

// SAFETY: The referent of a shared reference will never change unexpectedly and the table returned matches that returned by dereferencing
//         it by definition. The DowncastUnchecked implementation is also a simple cast via converting to/from a pointer and so should be
//         correct.
unsafe impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> GetDynDynTable<B> for &'a T {
    type DynTarget = T;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        B::get_dyn_dyn_table(*self)
    }
}

impl<'a, T: ?Sized> DowncastUnchecked<'a> for &'a T {
    type DowncastResult<D: ?Sized + 'a> = &'a D;

    unsafe fn downcast_unchecked<D: ?Sized + Pointee>(
        self,
        metadata: <D as Pointee>::Metadata,
    ) -> &'a D {
        // SAFETY: Safety invariants for this fn require that the provided metadata is valid for self. Since the input reference has the
        //         lifetime 'a and the returned reference also has lifetime 'a, this dereference does not extend the reference's lifetime
        //         and only serves to re-attach the metadata.
        unsafe { &*ptr::from_raw_parts(self as *const T as *const (), metadata) }
    }
}

// SAFETY: Since T is StableDeref, the results of its Deref implementation should meet the stability requirements and the table returned is
//         simply passed through from T's GetDynDynTable<B> implementation, which is unsafe itself and can be assumed to be correct. The
//         DowncastUnchecked implementation defers to the impl for &T::Target, so it should be correct.
unsafe impl<'a, B: ?Sized + DynDynBase, T: GetDynDynTable<B> + StableDeref + 'a> GetDynDynTable<B>
    for DynDynRef<'a, B, T>
where
    T::Target: Unsize<B>,
{
    type DynTarget = T::DynTarget;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        <T as GetDynDynTable<B>>::get_dyn_dyn_table(self.0)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B> + StableDeref + 'a> DowncastUnchecked<'a>
    for DynDynRef<'a, B, T>
where
    T::Target: Unsize<B>,
{
    type DowncastResult<D: ?Sized + 'a> = &'a D;

    unsafe fn downcast_unchecked<D: ?Sized + Pointee>(
        self,
        metadata: <D as Pointee>::Metadata,
    ) -> Self::DowncastResult<D> {
        // SAFETY: Just passing through to the implementation for &'a T.
        unsafe { <&T::Target as DowncastUnchecked>::downcast_unchecked(&**self.0, metadata) }
    }
}

// SAFETY: The referent of a mutable reference will never change unexpectedly and the table is returned by deferring to &T's implementation
//         and so should be correct. The DowncastUnchecked implementation is also a simple cast via converting to/from a pointer and so
//         should also be correct.
unsafe impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> GetDynDynTable<B> for &'a mut T {
    type DynTarget = T;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        B::get_dyn_dyn_table(*self)
    }
}

impl<'a, T: ?Sized> DowncastUnchecked<'a> for &'a mut T {
    type DowncastResult<D: ?Sized + 'a> = &'a mut D;

    unsafe fn downcast_unchecked<D: ?Sized + Pointee>(
        self,
        metadata: <D as Pointee>::Metadata,
    ) -> &'a mut D {
        // SAFETY: Safety invariants for this fn require that the provided metadata is valid for self. Since the input reference has the
        //         lifetime 'a and the returned reference also has lifetime 'a, this dereference does not extend the reference's lifetime
        //         and only serves to re-attach the metadata.
        unsafe { &mut *ptr::from_raw_parts_mut(self as *mut T as *mut (), metadata) }
    }
}

// SAFETY: Since T is StableDeref, the results of its Deref and DerefMut implementations should meet the stability requirements and the
//         table returned is simply passed through from T's GetDynDynTable<B> implementation, which is unsafe itself and can be assumed to
//         be correct. The DowncastUnchecked implementation defers to the impl for &mut T::Target, so it should be correct.
unsafe impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B> + StableDeref + DerefMut + 'a>
    GetDynDynTable<B> for DynDynRefMut<'a, B, T>
where
    T::Target: Unsize<B>,
{
    type DynTarget = T::Target;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        <T as GetDynDynTable<B>>::get_dyn_dyn_table(self.0)
    }
}

impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B> + StableDeref + DerefMut + 'a>
    DowncastUnchecked<'a> for DynDynRefMut<'a, B, T>
where
    T::Target: Unsize<B>,
{
    type DowncastResult<D: ?Sized + 'a> = &'a mut D;

    unsafe fn downcast_unchecked<D: ?Sized + Pointee>(
        self,
        metadata: <D as Pointee>::Metadata,
    ) -> Self::DowncastResult<D> {
        // SAFETY: Just passing through to the implementation for &'a mut T.
        unsafe {
            <&mut T::Target as DowncastUnchecked>::downcast_unchecked(&mut **self.0, metadata)
        }
    }
}

cfg_if! {
    if #[cfg(feature = "alloc")] {
        use alloc::boxed::Box;
        use alloc::sync::Arc;
        use alloc::rc::Rc;

        // SAFETY: Box<T> meets all Deref/DerefMut stability requirements and the table is retrieved by dereferencing it, which is correct
        //         by definition. The DowncastUnchecked implementation is also a simple cast via converting to/from a pointer and so should
        //         be correct.
        unsafe impl<B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> GetDynDynTable<B> for Box<T> {
            type DynTarget = T;

            fn get_dyn_dyn_table(&self) -> DynDynTable {
                B::get_dyn_dyn_table(&**self)
            }
        }

        impl<'a, T: ?Sized + 'a> DowncastUnchecked<'a>
            for Box<T>
        {
            type DowncastResult<D: ?Sized + 'a> = Box<D>;

            unsafe fn downcast_unchecked<D: ?Sized + Pointee>(self, metadata: <D as Pointee>::Metadata) -> Box<D> {
                // SAFETY: 1) NonNull::new_unchecked is fine since the raw pointer of a Box can never be null.
                //         2) Box::from_raw is fine since the fat pointer passed in has the same data pointer as what we got from
                //            Box::into_raw and the metadata pointer is guaranteed to be valid by this fn's safety invariants.
                unsafe {
                    Box::from_raw(
                        ptr::from_raw_parts_mut(Box::into_raw(self) as *mut (), metadata)
                    )
                }
            }
        }

        // SAFETY: Rc<T> meets all Deref/DerefMut stability requirements and the table is retrieved by dereferencing it, which is correct by
        //         definition. The DowncastUnchecked implementation is also a simple cast via converting to/from a pointer and so should be
        //         correct.
        unsafe impl<B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> GetDynDynTable<B> for Rc<T> {
            type DynTarget = T;

            fn get_dyn_dyn_table(&self) -> DynDynTable {
                B::get_dyn_dyn_table(&**self)
            }
        }

        impl<'a, T: ?Sized + 'a> DowncastUnchecked<'a>
            for Rc<T>
        {
            type DowncastResult<D: ?Sized + 'a> = Rc<D>;

            unsafe fn downcast_unchecked<D: ?Sized + Pointee>(self, metadata: <D as Pointee>::Metadata) -> Rc<D> {
                // SAFETY: 1) NonNull::new_unchecked is fine since the raw pointer of a Box can never be null.
                //         2) Rc::from_raw is fine since the fat pointer passed in has the same data pointer as what we got from
                //            Rc::into_raw and the metadata pointer is guaranteed to be valid by this fn's safety invariants.
                unsafe {
                    Rc::from_raw(
                        ptr::from_raw_parts(
                            Rc::into_raw(self) as *const (),
                            metadata,
                        ),
                    )
                }
            }
        }

        // SAFETY: Arc<T> meets all Deref/DerefMut stability requirements and the table is retrieved by dereferencing it, which is correct
        //         by definition. The DowncastUnchecked implementation is also a simple cast via converting to/from a pointer and so should
        //         be correct.
        unsafe impl<B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> GetDynDynTable<B> for Arc<T> {
            type DynTarget = T;

            fn get_dyn_dyn_table(&self) -> DynDynTable {
                B::get_dyn_dyn_table(&**self)
            }
        }

        impl<'a, T: ?Sized + 'a> DowncastUnchecked<'a>
            for Arc<T>
        {
            type DowncastResult<D: ?Sized + 'a> = Arc<D>;

            unsafe fn downcast_unchecked<D: ?Sized + Pointee>(self, metadata: <D as Pointee>::Metadata) -> Arc<D> {
                // SAFETY: Arc::from_raw is fine since the fat pointer passed in has the same data pointer as what we got from
                //         Arc::into_raw and the metadata pointer is guaranteed to be valid by this fn's safety invariants.
                unsafe {
                    Arc::from_raw(
                        ptr::from_raw_parts(Arc::into_raw(self) as *const (), metadata),
                    )
                }
            }
        }
    }
}
