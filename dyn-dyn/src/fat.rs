use crate::{DowncastUnchecked, DynDynBase, DynDynTable, DynTrait, GetDynDynTable};
use core::cmp::Ordering;
use core::fmt::{self, Display, Pointer};
use core::hash::{Hash, Hasher};
use core::marker::{PhantomData, Unsize};
use core::ops::CoerceUnsized;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, DynMetadata};
use stable_deref_trait::{CloneStableDeref, StableDeref};

/// A fat pointer to an object that can be downcast via the base trait object `B`.
///
/// Such a pointer will only perform a call to retrieve the [`DynDynTable`] of the referenced object once when created. Thereafter, the
/// cached table will be used for trait object metadata lookups. This effectively avoids the overhead of the repeated indirect calls to
/// retrieve the table at the cost of increasing the size of the pointer to the object.
#[derive(Debug)]
pub struct DynDynFat<B: ?Sized + DynDynBase, P> {
    ptr: P,
    table: DynDynTable,
    _base: PhantomData<fn(B) -> B>,
}

impl<B: ?Sized + DynDynBase, P: GetDynDynTable<B>> DynDynFat<B, P> {
    /// Creates a new fat pointer with the provided pointer and [`DynDynTable`].
    ///
    /// # Safety
    ///
    /// `table` must refer to the same table as a table that would be returned by calling [`GetDynDynTable::get_dyn_dyn_table`] on `ptr` at
    /// the time that this function is called.
    pub unsafe fn new_unchecked(ptr: P, table: DynDynTable) -> Self {
        DynDynFat {
            ptr,
            table,
            _base: PhantomData,
        }
    }

    /// Creates a new fat pointer wrapping the provided pointer. This will immediately get the pointer's [`DynDynTable`] by calling
    /// [`GetDynDynTable<B>::get_dyn_dyn_table`] and cache it for future use.
    pub fn new(ptr: P) -> Self {
        let table = ptr.get_dyn_dyn_table();

        // SAFETY: This table was just retrieved by calling get_dyn_dyn_table, so it's valid
        unsafe { Self::new_unchecked(ptr, table) }
    }

    /// Gets the [`DynDynTable`] of the object referenced by a fat pointer without dereferencing it.
    pub fn get_dyn_dyn_table(ptr: &Self) -> DynDynTable {
        ptr.table
    }

    /// Unwraps a fat pointer, returning the pointer originally used to construct it.
    pub fn unwrap(ptr: Self) -> P {
        ptr.ptr
    }
}

impl<B: ?Sized + DynDynBase, P: Deref> DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    /// Dereferences a fat pointer to produce a fat pointer with a reference.
    pub fn deref_fat(ptr: &Self) -> DynDynFat<B, &P::Target> {
        DynDynFat {
            ptr: ptr.ptr.deref(),
            table: ptr.table,
            _base: PhantomData,
        }
    }
}

impl<B: ?Sized + DynDynBase, P: DerefMut> DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    /// Dereferences a fat pointer to produce a fat pointer with a mutable reference.
    pub fn deref_mut_fat(ptr: &mut Self) -> DynDynFat<B, &mut P::Target> {
        DynDynFat {
            ptr: ptr.ptr.deref_mut(),
            table: ptr.table,
            _base: PhantomData,
        }
    }
}

impl<B: ?Sized + DynDynBase, P: Clone> DynDynFat<B, P> {
    /// Clones a fat pointer without verifying that the [`DynDynTable`] held by the new fat pointer is applicable to the cloned pointer.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the result of calling `ptr.clone()` will dereference to an object having the same concrete type as
    /// that pointed to by `ptr`.
    pub unsafe fn clone_unchecked(ptr: &Self) -> Self {
        DynDynFat {
            ptr: ptr.ptr.clone(),
            table: ptr.table,
            _base: PhantomData,
        }
    }
}

// SAFETY: The table returned by this implementation was retrieved from the pointer at the time the DynDynFat was created and DynDynFat does
//         not expose any way to mutate the pointer itself. Additionally, DynTarget is simply passed through from the pointer, so it must be
//         valid for that pointer.
unsafe impl<B: ?Sized + DynDynBase, P: GetDynDynTable<B>> GetDynDynTable<B> for DynDynFat<B, P> {
    type DynTarget = P::DynTarget;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        self.table
    }
}

impl<'a, B: ?Sized + DynDynBase, P: DowncastUnchecked<'a, B> + 'a> DowncastUnchecked<'a, B>
    for DynDynFat<B, P>
{
    type DowncastResult<D: ?Sized + 'a> = <P as DowncastUnchecked<'a, B>>::DowncastResult<D>;

    unsafe fn downcast_unchecked<D: ?Sized + DynTrait>(
        self,
        metadata: DynMetadata<D>,
    ) -> Self::DowncastResult<D> {
        self.ptr.downcast_unchecked(metadata)
    }
}

impl<B: ?Sized + DynDynBase, P: Deref> Deref for DynDynFat<B, P> {
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        self.ptr.deref()
    }
}

// SAFETY: DynDynFat implements deref() by dereferencing the wrapped pointer, so if that pointer is StableDeref then so is the DynDynFat
unsafe impl<B: ?Sized + DynDynBase, P: StableDeref> StableDeref for DynDynFat<B, P> {}

// SAFETY: DynDynFat implements clone() by cloning the wrapped pointer and implements deref() by dereferencing the wrapped pointer, so if
//         that pointer is CloneStableDeref then so is the DynDynFat
unsafe impl<B: ?Sized + DynDynBase, P: CloneStableDeref + GetDynDynTable<B>> CloneStableDeref
    for DynDynFat<B, P>
{
}

impl<B: ?Sized + DynDynBase, P: DerefMut> DerefMut for DynDynFat<B, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.deref_mut()
    }
}

impl<B: ?Sized + DynDynBase, P: Deref + Clone + GetDynDynTable<B>> Clone for DynDynFat<B, P> {
    fn clone(&self) -> Self {
        let ptr = self.ptr.clone();
        let table = if ptr::eq(ptr.deref(), self.ptr.deref()) {
            self.table
        } else {
            <P as GetDynDynTable<B>>::get_dyn_dyn_table(&ptr)
        };

        DynDynFat {
            ptr,
            table,
            _base: PhantomData,
        }
    }
}

impl<B: ?Sized + DynDynBase, P: Deref + Copy + GetDynDynTable<B>> Copy for DynDynFat<B, P> where
    P::Target: Unsize<B>
{
}

impl<B: ?Sized + DynDynBase, P: GetDynDynTable<B> + Default> Default for DynDynFat<B, P> {
    fn default() -> Self {
        DynDynFat::new(Default::default())
    }
}

impl<B: ?Sized + DynDynBase, P: GetDynDynTable<B>> From<P> for DynDynFat<B, P> {
    fn from(ptr: P) -> Self {
        DynDynFat::new(ptr)
    }
}

impl<B: ?Sized + DynDynBase, P: Deref> AsRef<P> for DynDynFat<B, P> {
    fn as_ref(&self) -> &P {
        &self.ptr
    }
}

impl<B1: ?Sized + DynDynBase, B2: ?Sized + DynDynBase, P1: Deref, P2: Deref>
    PartialEq<DynDynFat<B2, P2>> for DynDynFat<B1, P1>
where
    P1::Target: Unsize<B1> + PartialEq<P2::Target>,
    P2::Target: Unsize<B2>,
{
    fn eq(&self, other: &DynDynFat<B2, P2>) -> bool {
        PartialEq::eq(&*self.ptr, &*other.ptr)
    }
}

impl<B: ?Sized + DynDynBase, P: Deref> Eq for DynDynFat<B, P> where P::Target: Unsize<B> + Eq {}

impl<B: ?Sized + DynDynBase, P: Deref> Hash for DynDynFat<B, P>
where
    P::Target: Unsize<B> + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&*self.ptr, state)
    }
}

impl<B1: ?Sized + DynDynBase, B2: ?Sized + DynDynBase, P1: Deref, P2: Deref>
    PartialOrd<DynDynFat<B2, P2>> for DynDynFat<B1, P1>
where
    P1::Target: Unsize<B1> + PartialOrd<P2::Target>,
    P2::Target: Unsize<B2>,
{
    fn partial_cmp(&self, other: &DynDynFat<B2, P2>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&*self.ptr, &*other.ptr)
    }
}

impl<B: ?Sized + DynDynBase, P: Deref> Ord for DynDynFat<B, P>
where
    P::Target: Unsize<B> + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&*self.ptr, &*other.ptr)
    }
}

impl<B: ?Sized + DynDynBase, P1: Deref + CoerceUnsized<P2>, P2: Deref>
    CoerceUnsized<DynDynFat<B, P2>> for DynDynFat<B, P1>
where
    P1::Target: Unsize<B>,
    P2::Target: Unsize<B>,
{
}

impl<B: ?Sized + DynDynBase, P: Display> Display for DynDynFat<B, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ptr)
    }
}

impl<B: ?Sized + DynDynBase, P: Pointer> Pointer for DynDynFat<B, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}
