#![doc = include_str!("../../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![feature(coerce_unsized)]
#![feature(const_convert)]
#![feature(const_nonnull_new)]
#![feature(const_option)]
#![feature(const_trait_impl)]
#![feature(const_type_id)]
#![cfg_attr(feature = "dynamic-names", feature(const_type_name))]
#![feature(doc_auto_cfg)]
#![feature(ptr_metadata)]
#![feature(unsize)]

#[cfg(feature = "alloc")]
extern crate alloc;

/// Declares a trait as being a base trait for downcasting.
///
/// This macro marks a trait as being a base for dynamic trait object downcasting. All `impl` blocks for this trait will need to use the
/// [`#[dyn_dyn_derived]`](dyn_dyn_derived) attribute to declare what traits they wish to expose.
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
/// # use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_derived};
/// #[dyn_dyn_base]
/// trait Base {}
/// trait Trait {}
///
/// struct Struct;
///
/// #[dyn_dyn_derived(Trait)]
/// impl Base for Struct {}
/// impl Trait for Struct {}
///
/// fn downcast(r: &dyn Base) -> Option<&dyn Trait> {
///     dyn_dyn_cast!(Base => Trait, r)
/// }
///
/// fn downcast_mut(r: &mut dyn Base) -> Option<&mut dyn Trait> {
///     dyn_dyn_cast!(mut Base => Trait, r)
/// }
///
/// fn downcast_with_auto(r: &(dyn Base + Send)) -> Option<&(dyn Trait + Send)> {
///     dyn_dyn_cast!(Base + Send => Trait + Send, r)
/// }
///
/// fn main() {
///     let mut s = Struct;
///
///     assert!(downcast(&s).is_some());
///     assert!(downcast_mut(&mut s).is_some());
///     assert!(downcast_with_auto(&s).is_some());
/// }
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
/// # use dyn_dyn::{dyn_dyn_base, dyn_dyn_derived};
/// #[dyn_dyn_base]
/// trait Base {}
///
/// #[derive(Debug)]
/// struct Struct;
///
/// #[dyn_dyn_derived(Debug)]
/// impl Base for Struct {}
/// ```
///
/// ```rust
/// # use dyn_dyn::{dyn_dyn_base, dyn_dyn_derived};
/// #[dyn_dyn_base]
/// trait Base {}
/// trait Trait<T> {}
///
/// struct Struct<T>(T);
///
/// impl<T> Trait<T> for Struct<T> {}
///
/// #[dyn_dyn_derived(Trait<T>)]
/// impl<T: 'static> Base for Struct<T> {}
/// ```
pub use dyn_dyn_macros::dyn_dyn_derived;

pub use fat::DynDynFat;

mod dyn_trait;
mod fat;

#[doc(hidden)]
pub mod internal;

use core::any::TypeId;
use core::fmt::{self, Debug};
use core::marker::Unsize;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use crate::dyn_trait::{AnyDynMetadata, DynInfo, DynTrait};
use internal::*;

/// An entry in a concrete type's table of downcast-exposed traits.
///
/// Each entry represents a single trait object that the concrete type in question can be downcast to. Note that entries will only appear
/// for bare trait object types, i.e. `dyn Trait`. Trait objects with extra marker types, e.g. `dyn Trait + Send`, are handled specially
/// by the [`dyn_dyn_cast!`] macro and do not appear in a concrete type's trait table.
pub struct DynDynTableEntry {
    ty: DynInfo,
    meta: AnyDynMetadata,
}

impl DynDynTableEntry {
    #[doc(hidden)]
    pub const unsafe fn new<
        T,
        D: ?Sized + ~const DynTrait + 'static,
        F: ~const FnOnce(*const T) -> *const D,
    >(
        f: F,
    ) -> DynDynTableEntry {
        DynDynTableEntry {
            ty: DynInfo::of::<D>(),
            meta: D::meta_for_ty(f),
        }
    }

    /// Gets the [`TypeId`] of the trait object corresponding to this entry.
    pub fn type_id(&self) -> TypeId {
        self.ty.type_id()
    }

    /// Gets a human-readable name representing the trait object corresponding to this entry.
    #[cfg(feature = "dynamic-names")]
    pub fn type_name(&self) -> &'static str {
        self.ty.name()
    }
}

impl Debug for DynDynTableEntry {
    #[cfg(feature = "dynamic-names")]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DynDynTableEntry(<{}>: {:?})",
            self.type_name(),
            self.meta
        )
    }

    #[cfg(not(feature = "dynamic-names"))]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DynDynTableEntry({:?}: {:?})", self.type_id(), self.meta)
    }
}

/// A table of trait object types that a concrete type can be downcast to.
#[derive(Debug, Clone, Copy)]
pub struct DynDynTable {
    traits: &'static [DynDynTableEntry],
}

impl DynDynTable {
    fn find(&self, type_id: TypeId) -> Option<AnyDynMetadata> {
        self.traits
            .iter()
            .find(|&entry| entry.ty.type_id() == type_id)
            .map(|entry| entry.meta)
    }

    unsafe fn find_dyn<D: DynTrait + ?Sized>(&self, data: NonNull<()>) -> Option<NonNull<D>> {
        self.find(TypeId::of::<D>())
            .map(|meta| D::ptr_from_parts(data, meta))
    }

    /// Returns a reference to the slice of entries in this table
    pub fn into_slice(self) -> &'static [DynDynTableEntry] {
        self.traits
    }

    #[doc(hidden)]
    pub const fn new(traits: &'static [DynDynTableEntry]) -> DynDynTable {
        DynDynTable { traits }
    }
}

impl IntoIterator for DynDynTable {
    type Item = &'static DynDynTableEntry;
    type IntoIter = DynDynTableIterator;

    fn into_iter(self) -> Self::IntoIter {
        DynDynTableIterator(self.traits.iter())
    }
}

/// An iterator returning all entries in a [`DynDynTable`].
pub struct DynDynTableIterator(core::slice::Iter<'static, DynDynTableEntry>);

impl Iterator for DynDynTableIterator {
    type Item = &'static DynDynTableEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// A pointer that can have its pointee dynamically cast to other trait types via the base trait object type `B`.
///
/// # Safety
///
/// Any reference returned by [`DynDyn::deref_dyn_dyn`] must have been received by calling [`Deref::deref`]. The table returned by
/// [`DynDyn::deref_dyn_dyn`] must correspond to the correct concrete type matching the returned reference. This table must also not change
/// for any subsequent calls to [`DynDyn::deref_dyn_dyn`] for the lifetime of this pointer.
pub unsafe trait DynDyn<B: ?Sized + DynDynBase>: Deref {
    /// Dereferences this pointer, returning a reference to its pointee and a [`DynDynTable`] corresponding to the pointee's concrete type.
    fn deref_dyn_dyn(&self) -> (&B, DynDynTable);
}

// SAFETY: The DynDynTable returned comes from calling get_dyn_dyn_table on the returned reference. The concrete type of the pointee of a
//         reference also cannot change for the lifetime of that reference, so the table returned should be stable.
unsafe impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DynDyn<B> for &'a T {
    #[inline]
    fn deref_dyn_dyn(&self) -> (&B, DynDynTable) {
        let tgt = &**self;
        let table = B::get_dyn_dyn_table(tgt);

        (tgt, table)
    }
}

// SAFETY: The DynDynTable returned comes from calling get_dyn_dyn_table on the returned reference. The concrete type of the pointee of a
//         reference also cannot change for the lifetime of that reference, so the table returned should be stable.
unsafe impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DynDyn<B> for &'a mut T {
    fn deref_dyn_dyn(&self) -> (&B, DynDynTable) {
        let tgt = &**self;
        let table = B::get_dyn_dyn_table(tgt);

        (tgt, table)
    }
}

/// A pointer that can have its mutable pointee dynamically cast to other trait types via the base trait object type `B`.
///
/// # Safety
///
/// Any reference returned by [`DynDynMut::deref_mut_dyn_dyn`] must have been received by calling [`DerefMut::deref_mut`]. The table
/// returned by [`DynDynMut::deref_mut_dyn_dyn`] must correspond to the correct concrete type matching the returned reference.
///
/// Additionally, the [`DynDynTable`] value returned must not change for the lifetime of this pointer and must match the table returned by
/// calling [`DynDyn::deref_dyn_dyn`].
pub unsafe trait DynDynMut<B: ?Sized + DynDynBase>: DynDyn<B> + DerefMut {
    /// Dereferences this pointer, returning a mutable reference to its pointee and a [`DynDynTable`] corresponding to the pointee's
    /// concrete type.
    fn deref_mut_dyn_dyn(&mut self) -> (&mut B, DynDynTable);
}

// SAFETY: The DynDynTable returned comes from calling get_dyn_dyn_table on the returned reference. The concrete type of the pointee of a
//         reference also cannot change for the lifetime of that reference, so the table returned should be stable.
unsafe impl<'a, B: ?Sized + DynDynBase, T: ?Sized + Unsize<B>> DynDynMut<B> for &'a mut T {
    #[inline]
    fn deref_mut_dyn_dyn(&mut self) -> (&mut B, DynDynTable) {
        let tgt = &mut **self;
        let table = B::get_dyn_dyn_table(tgt);

        (tgt, table)
    }
}
