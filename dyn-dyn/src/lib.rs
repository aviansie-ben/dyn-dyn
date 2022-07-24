#![feature(const_convert)]
#![feature(const_nonnull_new)]
#![feature(const_option)]
#![feature(const_trait_impl)]
#![feature(const_type_id)]
#![cfg_attr(feature = "dynamic-names", feature(const_type_name))]
#![feature(ptr_metadata)]

mod dyn_trait;

#[doc(hidden)]
pub mod internal;

use std::any::TypeId;
use std::ptr::NonNull;

pub use dyn_trait::DynInfo;
use internal::*;
use crate::dyn_trait::{AnyDynMetadata, DynTrait};

#[derive(Debug)]
pub struct DynDynTableEntry {
    ty: DynInfo,
    send_ty: TypeId,
    sync_ty: TypeId,
    send_sync_ty: TypeId,
    meta: AnyDynMetadata
}

impl DynDynTableEntry {
    #[doc(hidden)]
    pub const unsafe fn new<
        T,
        D: ?Sized + ~const DynTrait + 'static,
        DSend: ?Sized + ~const DynTrait + 'static,
        DSync: ?Sized + ~const DynTrait + 'static,
        DSendSync: ?Sized + ~const DynTrait + 'static,
        F: ~const FnOnce(*const T) -> *const D
    >(f: F) -> DynDynTableEntry {
        DynDynTableEntry {
            ty: DynInfo::of::<D>(),
            send_ty: TypeId::of::<DSend>(),
            sync_ty: TypeId::of::<DSync>(),
            send_sync_ty: TypeId::of::<DSendSync>(),
            meta: D::meta_for_ty(f)
        }
    }
}

#[derive(Debug)]
pub struct DynDynTable {
    traits: &'static [DynDynTableEntry]
}

impl DynDynTable {
    fn find(&self, type_id: TypeId, send: bool, sync: bool) -> Option<AnyDynMetadata> {
        self.traits.iter().find(|&entry| {
            #[allow(clippy::if_same_then_else)]
            #[allow(clippy::needless_bool)]
            if entry.ty.type_id() == type_id {
                true
            } else if send && entry.send_ty == type_id {
                true
            } else if sync && entry.sync_ty == type_id {
                true
            } else if send && sync && entry.send_sync_ty == type_id {
                true
            } else {
                false
            }
        }).map(|entry| entry.meta)
    }

    unsafe fn find_dyn<D: DynTrait + ?Sized>(&self, data: NonNull<()>, send: bool, sync: bool) -> Option<NonNull<D>> {
        self.find(TypeId::of::<D>(), send, sync).map(|meta| D::ptr_from_parts(data, meta))
    }

    #[doc(hidden)]
    pub const fn new(traits: &'static [DynDynTableEntry]) -> DynDynTable {
        DynDynTable { traits }
    }
}

mod sealed {
    pub trait Sealed<B: ?Sized> {}

    impl<B: ?Sized, T: crate::internal::DynDynImpl<B> + ?Sized> Sealed<B> for T {}
}

pub trait DynDyn<B: ?Sized>: sealed::Sealed<B> {
    #[must_use]
    fn try_downcast<D: DynTrait + ?Sized>(&self) -> Option<&D>;

    #[must_use]
    fn try_downcast_mut<D: DynTrait + ?Sized>(&mut self) -> Option<&mut D>;
}

impl<B: ?Sized, T: DynDynImpl<B> + ?Sized> DynDyn<B> for T {
    fn try_downcast<D: DynTrait + ?Sized>(&self) -> Option<&D> {
        unsafe {
            self.get_dyn_dyn_table().find_dyn(NonNull::from(self).cast(), Self::IS_SEND, Self::IS_SYNC).map(|ptr| &*ptr.as_ptr())
        }
    }

    fn try_downcast_mut<D: DynTrait + ?Sized>(&mut self) -> Option<&mut D> {
        unsafe {
            self.get_dyn_dyn_table().find_dyn(NonNull::from(self).cast(), Self::IS_SEND, Self::IS_SYNC).map(|ptr| &mut *ptr.as_ptr())
        }
    }
}
