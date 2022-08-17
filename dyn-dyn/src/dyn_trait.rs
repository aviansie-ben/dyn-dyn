use core::marker::Unsize;
use core::ptr::{DynMetadata, NonNull, Pointee};
use core::{mem, ptr};

#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
pub struct AnyDynMetadata(*const ());

#[allow(clippy::missing_safety_doc, missing_docs)] // This module is marked doc(hidden)
impl AnyDynMetadata {
    pub const unsafe fn downcast<T: DynTrait + ?Sized>(self) -> DynMetadata<T> {
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

pub trait DynTrait: private::Sealed {
    fn ptr_into_parts(ptr: NonNull<Self>) -> (NonNull<()>, DynMetadata<Self>);
    fn ptr_from_parts(data: NonNull<()>, meta: DynMetadata<Self>) -> NonNull<Self>;

    fn meta_for_ty<U: Unsize<Self>>() -> AnyDynMetadata;
}

impl<T: Pointee<Metadata = DynMetadata<T>> + ?Sized> const DynTrait for T {
    fn ptr_into_parts(ptr: NonNull<Self>) -> (NonNull<()>, DynMetadata<T>) {
        (ptr.cast(), ptr::metadata(ptr.as_ptr()))
    }

    fn ptr_from_parts(data: NonNull<()>, meta: DynMetadata<T>) -> NonNull<Self> {
        // SAFETY: If data is not null, then the result of attaching metadata to it is not null either
        unsafe { NonNull::new_unchecked(ptr::from_raw_parts_mut(data.as_ptr(), meta)) }
    }

    fn meta_for_ty<U: Unsize<Self>>() -> AnyDynMetadata {
        ptr::metadata(ptr::null::<U>() as *const Self).into()
    }
}

mod private {
    use core::ptr::{DynMetadata, Pointee};

    pub trait Sealed {}

    impl<T: Pointee<Metadata = DynMetadata<T>> + ?Sized> Sealed for T {}
}
