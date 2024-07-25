use core::ptr::{self, DynMetadata, NonNull, Pointee};

/// A type whose pointer metadata can be stored in a [`DynDynTable`](crate::DynDynTable).
pub trait DynDynCastTarget: private::Sealed {
    /// The root type that the trait object metadata is for.
    type Root: ?Sized;

    /// Splits a fat pointer into its data pointer and metadata.
    fn ptr_into_parts(ptr: NonNull<Self>) -> (NonNull<()>, DynMetadata<Self::Root>);

    /// Combines a data pointer with the provided metadata to produce a fat pointer.
    fn ptr_from_parts(data: NonNull<()>, meta: DynMetadata<Self::Root>) -> NonNull<Self>;
}

impl<M: ?Sized, T: Pointee<Metadata = DynMetadata<M>> + ?Sized> DynDynCastTarget for T {
    type Root = M;

    fn ptr_into_parts(ptr: NonNull<Self>) -> (NonNull<()>, DynMetadata<M>) {
        (ptr.cast(), ptr::metadata(ptr.as_ptr()))
    }

    fn ptr_from_parts(data: NonNull<()>, meta: DynMetadata<M>) -> NonNull<Self> {
        // SAFETY: If data is not null, then the result of attaching metadata to it is not null either
        unsafe { NonNull::new_unchecked(ptr::from_raw_parts_mut(data.as_ptr(), meta)) }
    }
}

mod private {
    use core::ptr::{DynMetadata, Pointee};

    pub trait Sealed {}

    impl<M: ?Sized, T: Pointee<Metadata = DynMetadata<M>> + ?Sized> Sealed for T {}
}
