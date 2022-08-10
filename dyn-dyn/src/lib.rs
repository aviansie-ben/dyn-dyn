#![cfg_attr(not(feature = "std"), no_std)]
#![feature(const_convert)]
#![feature(const_nonnull_new)]
#![feature(const_option)]
#![feature(const_trait_impl)]
#![feature(const_type_id)]
#![cfg_attr(feature = "dynamic-names", feature(const_type_name))]
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

use crate::dyn_trait::{AnyDynMetadata, DynTrait};
pub use dyn_trait::DynInfo;
use internal::*;

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
}

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

pub unsafe trait DynDyn<'a, B: ?Sized + DynDynBase> {
    type DerefTarget: ?Sized;

    fn deref_dyn_dyn(&self) -> (&B, DynDynTable);
}

unsafe impl<'a, B: ?Sized + DynDynBase, T: Deref> DynDyn<'a, B> for T
where
    T::Target: Unsize<B> + 'a,
{
    type DerefTarget = T::Target;

    #[inline]
    fn deref_dyn_dyn(&self) -> (&B, DynDynTable) {
        let tgt = &**self;
        let table = B::get_dyn_dyn_table(tgt);

        (tgt, table)
    }
}

pub unsafe trait DynDynMut<'a, B: ?Sized + DynDynBase>: DynDyn<'a, B> {
    fn deref_mut_dyn_dyn(&mut self) -> (&mut B, DynDynTable);
}

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

pub unsafe trait StableDynDyn<'a, B: ?Sized + DynDynBase>: DynDyn<'a, B> {}

unsafe impl<'a, B: ?Sized + DynDynBase, T: DynDyn<'a, B> + StableDeref> StableDynDyn<'a, B> for T {}
