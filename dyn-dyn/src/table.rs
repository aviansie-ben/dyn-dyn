use crate::dyn_trait::{AnyDynMetadata, DynTrait};
use cfg_if::cfg_if;
use core::any::TypeId;
use core::fmt::{self, Debug};
use core::ptr::DynMetadata;

#[cfg(doc)]
use crate::dyn_dyn_cast;

cfg_if! {
    if #[cfg(feature = "dynamic-names")] {
        type TypeName = &'static str;

        const fn type_name<T: ?Sized>() -> TypeName {
            core::any::type_name::<T>()
        }
    } else {
        #[derive(Debug, Clone, Copy)]
        struct TypeName;

        const fn type_name<T: ?Sized>() -> TypeName { TypeName }
    }
}

#[derive(Debug, Clone, Copy)]
struct DynInfo(TypeId, TypeName);

impl DynInfo {
    pub const fn of<T: 'static + ?Sized>() -> DynInfo {
        DynInfo(TypeId::of::<T>(), type_name::<T>())
    }

    pub fn type_id(self) -> TypeId {
        self.0
    }

    #[cfg(feature = "dynamic-names")]
    pub fn name(self) -> &'static str {
        self.1
    }
}

impl PartialEq for DynInfo {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for DynInfo {}

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
    /// Finds the metadata corresponding to the type with the provided [`TypeId`] in this table or `None` if no such metadata is present.
    pub fn find_untyped(&self, type_id: TypeId) -> Option<AnyDynMetadata> {
        self.traits
            .iter()
            .find(|&entry| entry.ty.type_id() == type_id)
            .map(|entry| entry.meta)
    }

    /// Finds the metadata corresponding to the trait `D` in this table or `None` if no such metadata is present.
    pub fn find<D: ?Sized + DynTrait + 'static>(&self) -> Option<DynMetadata<D>> {
        self.find_untyped(TypeId::of::<D>()).map(|meta| {
            // SAFETY: This metadata corresponds to the trait D, so we can downcast it
            unsafe { meta.downcast() }
        })
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
