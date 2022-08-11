#![doc = include_str!("../../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]
#![warn(clippy::undocumented_unsafe_blocks)]
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

mod dyn_trait;
mod fat;

pub use fat::DynDynFat;

#[doc(hidden)]
pub mod internal;

use core::any::TypeId;
use core::marker::Unsize;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use stable_deref_trait::StableDeref;

use crate::dyn_trait::{AnyDynMetadata, DynInfo, DynTrait};
use internal::*;

/// An entry in a concrete type's table of downcast-exposed traits.
///
/// Each entry represents a single trait object that the concrete type in question can be downcast to. Note that entries will only appear
/// for bare trait object types, i.e. `dyn Trait`. Trait objects with extra marker types, e.g. `dyn Trait + Send`, are handled specially
/// by the [`dyn_dyn_cast!`](dyn_dyn_macros::dyn_dyn_cast!) macro and do not appear in a concrete type's trait table.
#[derive(Debug)]
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
/// [`DynDyn::deref_dyn_dyn`] must correspond to the correct concrete type matching the returned reference.
pub unsafe trait DynDyn<'a, B: ?Sized + DynDynBase>: Deref {
    /// Dereferences this pointer, returning a reference to its pointee and a [`DynDynTable`] corresponding to the pointee's concrete type.
    fn deref_dyn_dyn(&self) -> (&B, DynDynTable);
}

// SAFETY: The DynDynTable returned comes from calling get_dyn_dyn_table on the returned reference.
unsafe impl<'a, B: ?Sized + DynDynBase, T: Deref> DynDyn<'a, B> for T
where
    T::Target: Unsize<B> + 'a,
{
    #[inline]
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
pub unsafe trait DynDynMut<'a, B: ?Sized + DynDynBase>: DynDyn<'a, B> + DerefMut {
    /// Dereferences this pointer, returning a mutable reference to its pointee and a [`DynDynTable`] corresponding to the pointee's
    /// concrete type.
    fn deref_mut_dyn_dyn(&mut self) -> (&mut B, DynDynTable);
}

// SAFETY: The DynDynTable returned comes from calling get_dyn_dyn_table on the returned mutable reference.
unsafe impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B> + DerefMut> DynDynMut<'a, B> for T
where
    T::Target: Unsize<B> + 'a,
{
    #[inline]
    fn deref_mut_dyn_dyn(&mut self) -> (&mut B, DynDynTable) {
        let tgt = &mut **self;
        let table = B::get_dyn_dyn_table(tgt);

        (tgt, table)
    }
}

/// A pointer that can have its pointee dynamically cast to other trait types via the base trait object type `B` and for which the
/// [`DynDynTable`] returned by dereferencing it is stable.
///
/// # Safety
///
/// The [`DynDynTable`] values returned by [`DynDyn::deref_dyn_dyn`] must not change for the lifetime of this pointer. If [`DynDynMut`] is
/// implemented, the [`DynDynTable`] values returned by [`DynDynMut::deref_mut_dyn_dyn`] must not change either and must match the tables
/// returned by [`DynDyn::deref_dyn_dyn`].
pub unsafe trait StableDynDyn<'a, B: ?Sized + DynDynBase>: DynDyn<'a, B> {}

// SAFETY: Since the pointer itself must be stable, the underlying type of the pointee cannot change. Since the tables returned by the
//         DynDyn and DynDynMut implementations must have come from dereferencing the pointer, it follows that these tables should also be
//         stable.
unsafe impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B> + StableDeref> StableDynDyn<'a, B> for T {}
