use crate::cast_target::DynDynCastTarget;
use cfg_if::cfg_if;
use core::any::TypeId;
use core::fmt::{self, Debug};
use core::marker::Unsize;
use core::mem;
use core::ptr::DynMetadata;

#[cfg(doc)]
use crate::dyn_dyn_cast;

/// An untyped metadata that corresponds to the metadata that would be used for a trait object.
#[derive(Debug, Clone, Copy)]
pub struct AnyDynMetadata(*const ());

impl AnyDynMetadata {
    /// Downcasts this untyped metadata into typed metadata for a trait object referring to a particular trait.
    ///
    /// # Safety
    ///
    /// This untyped metadata must have originally been constructed by converting a `DynMetadata<T>`.
    pub const unsafe fn downcast<T: DynDynCastTarget + ?Sized>(self) -> DynMetadata<T> {
        mem::transmute(self.0)
    }
}

impl<T: ?Sized> const From<DynMetadata<T>> for AnyDynMetadata {
    fn from(meta: DynMetadata<T>) -> Self {
        // SAFETY: There are no invalid values for *const (), so transmuting to it should never cause UB if DynMetadata<T> is of the same
        //         size. The only valid usage of this *const () is then to transmute it back to DynMetadata<T>. While this definitely makes
        //         assumptions about how the standard library implements DynMetadata (which is unfortunate), we should fail to compile if
        //         that changes rather than invoking UB.
        unsafe { AnyDynMetadata(mem::transmute(meta)) }
    }
}

// SAFETY: Since DynMetadata<T>: Send for all T, AnyDynMetadata should also be Send
unsafe impl Send for AnyDynMetadata {}

// SAFETY: Since DynMetadata<T>: Sync for all T, AnyDynMetadata should also be Sync
unsafe impl Sync for AnyDynMetadata {}

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
    pub const fn new<T: Unsize<D>, D: ?Sized + ~const DynDynCastTarget + 'static>(
    ) -> DynDynTableEntry {
        DynDynTableEntry {
            ty: DynInfo::of::<D>(),
            meta: D::meta_for_ty::<T>().into(),
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
    pub fn find<D: ?Sized + DynDynCastTarget + 'static>(&self) -> Option<DynMetadata<D>> {
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
