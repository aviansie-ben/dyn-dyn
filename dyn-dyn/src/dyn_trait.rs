use cfg_if::cfg_if;
use std::any::TypeId;
use std::ptr::{DynMetadata, NonNull, Pointee};
use std::{mem, ptr};

#[derive(Debug, Clone, Copy)]
pub struct AnyDynMetadata(DynMetadata<()>);

impl AnyDynMetadata {
    pub const unsafe fn downcast<T: DynTrait + ?Sized>(self) -> DynMetadata<T> {
        mem::transmute(self.0)
    }
}

impl<T: ?Sized> const From<DynMetadata<T>> for AnyDynMetadata {
    fn from(meta: DynMetadata<T>) -> Self {
        unsafe { AnyDynMetadata(mem::transmute(meta)) }
    }
}

cfg_if! {
    if #[cfg(feature = "dynamic-names")] {
        type TypeName = &'static str;

        const fn type_name<T: ?Sized>() -> TypeName {
            std::any::type_name::<T>()
        }
    } else {
        #[derive(Debug, Clone, Copy)]
        struct TypeName;

        const fn type_name<T: ?Sized>() -> TypeName { TypeName }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DynInfo(TypeId, TypeName);

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

pub trait DynTrait: private::Sealed + 'static {
    fn ptr_into_parts(ptr: NonNull<Self>) -> (NonNull<()>, AnyDynMetadata);
    unsafe fn ptr_from_parts(data: NonNull<()>, meta: AnyDynMetadata) -> NonNull<Self>;

    unsafe fn meta_for_ty<U, F: ~const FnOnce(*const U) -> *const Self>(f: F) -> AnyDynMetadata;
}

impl<T: Pointee<Metadata = DynMetadata<T>> + ?Sized + 'static> const DynTrait for T {
    fn ptr_into_parts(ptr: NonNull<Self>) -> (NonNull<()>, AnyDynMetadata) {
        (ptr.cast(), ptr::metadata(ptr.as_ptr()).into())
    }

    unsafe fn ptr_from_parts(data: NonNull<()>, meta: AnyDynMetadata) -> NonNull<Self> {
        NonNull::new_unchecked(ptr::from_raw_parts_mut(data.as_ptr(), meta.downcast()))
    }

    unsafe fn meta_for_ty<U, F: ~const FnOnce(*const U) -> *const Self>(f: F) -> AnyDynMetadata {
        Self::ptr_into_parts(NonNull::new(f(NonNull::dangling().as_ptr()) as *mut _).unwrap()).1
    }
}

mod private {
    use std::ptr::{DynMetadata, Pointee};

    pub trait Sealed {}

    impl<T: Pointee<Metadata = DynMetadata<T>> + ?Sized + 'static> Sealed for T {}
}
