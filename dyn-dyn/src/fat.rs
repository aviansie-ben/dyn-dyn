use crate::{DynDyn, DynDynBase, DynDynMut, DynDynTable};
use core::cmp::Ordering;
use core::fmt::{self, Display, Pointer};
use core::hash::{Hash, Hasher};
use core::marker::{PhantomData, Unsize};
use core::ops::CoerceUnsized;
use core::ops::{Deref, DerefMut};
use core::ptr;
use stable_deref_trait::{CloneStableDeref, StableDeref};

/// A fat pointer to an object that can be downcast via the base trait object `B`.
///
/// Such a pointer will only perform a call to retrieve the [`DynDynTable`] of the referenced object once when created. Thereafter, the
/// cached table will be used for trait object metadata lookups. This effectively avoids the overhead of the repeated indirect calls to
/// retrieve the table at the cost of increasing the size of the pointer to the object.
#[derive(Debug)]
pub struct DynDynFat<B: ?Sized + DynDynBase, P: Deref>
where
    P::Target: Unsize<B>,
{
    ptr: P,
    table: DynDynTable,
    _base: PhantomData<fn(B) -> B>,
}

impl<B: ?Sized + DynDynBase, P: Deref> DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    /// Creates a new fat pointer wrapping the provided pointer. This will immediately dereference the provided pointer to get the
    /// referenced object's [`DynDynTable`].
    ///
    /// # Safety
    ///
    /// The caller must guarantee that as long as this fat pointer is live, `ptr` will dereference to an object of the same concrete type.
    /// Additionally, if this fat pointer is cloned, the same guarantee must apply to the result of `ptr.clone()` for the lifespan of the
    /// cloned fat pointer.
    pub unsafe fn new_unchecked(ptr: P) -> Self {
        let table = DynDyn::<B>::deref_dyn_dyn(&&*ptr).1;

        DynDynFat {
            ptr,
            table,
            _base: PhantomData,
        }
    }

    /// Dereferences a fat pointer to produce a fat pointer with a reference.
    pub fn deref_fat(ptr: &Self) -> DynDynFat<B, &P::Target> {
        DynDynFat {
            ptr: ptr.ptr.deref(),
            table: ptr.table,
            _base: PhantomData,
        }
    }

    /// Unwraps a fat pointer, returning the pointer originally used to construct it.
    pub fn unwrap(ptr: Self) -> P {
        ptr.ptr
    }

    /// Gets the [`DynDynTable`] of the object referenced by a fat pointer without dereferencing it.
    pub fn get_dyn_dyn_table(ptr: &Self) -> DynDynTable {
        ptr.table
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

impl<B: ?Sized + DynDynBase, P: StableDeref> DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    /// Creates a new fat pointer wrapping the provided pointer. This will immediately dereference the provided pointer to get the
    /// referenced object's [`DynDynTable`].
    pub fn new(ptr: P) -> Self {
        // SAFETY: Since the provided pointer implements StableDeref, it must always dereference to the same object whose concrete type
        //         cannot change for the life of this fat pointer.
        unsafe { Self::new_unchecked(ptr) }
    }
}

impl<B: ?Sized + DynDynBase, P: Deref + Clone> DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
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

impl<B: ?Sized + DynDynBase, P: Deref> Deref for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        self.ptr.deref()
    }
}

// SAFETY: All APIs for creating a DynDynFat either guarantee stable deref or are unsafe and require the caller to assert that this
//         DynDynFat's table meets the requirements for this impl
unsafe impl<B: ?Sized + DynDynBase, P: Deref> DynDyn<B> for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    fn deref_dyn_dyn(&self) -> (&B, DynDynTable) {
        (&*self.ptr, self.table)
    }
}

// SAFETY: All APIs for creating a DynDynFat either guarantee stable deref or are unsafe and require the caller to assert that this
//         DynDynFat's table meets the requirements for this impl
unsafe impl<B: ?Sized + DynDynBase, P: DerefMut> DynDynMut<B> for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    fn deref_mut_dyn_dyn(&mut self) -> (&mut B, DynDynTable) {
        (&mut *self.ptr, self.table)
    }
}

// SAFETY: DynDynFat implements deref() by dereferencing the wrapped pointer, so if that pointer is StableDeref then so is the DynDynFat
unsafe impl<B: ?Sized + DynDynBase, P: StableDeref> StableDeref for DynDynFat<B, P> where
    P::Target: Unsize<B>
{
}

// SAFETY: DynDynFat implements clone() by cloning the wrapped pointer and implements deref() by dereferencing the wrapped pointer, so if
//         that pointer is CloneStableDeref then so is the DynDynFat
unsafe impl<B: ?Sized + DynDynBase, P: CloneStableDeref> CloneStableDeref for DynDynFat<B, P> where
    P::Target: Unsize<B>
{
}

impl<B: ?Sized + DynDynBase, P: DerefMut> DerefMut for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.deref_mut()
    }
}

impl<B: ?Sized + DynDynBase, P: Deref + Clone> Clone for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    fn clone(&self) -> Self {
        let ptr = self.ptr.clone();
        let table = if ptr::eq(ptr.deref(), self.ptr.deref()) {
            self.table
        } else {
            DynDyn::<B>::deref_dyn_dyn(&&*ptr).1
        };

        DynDynFat {
            ptr,
            table,
            _base: PhantomData,
        }
    }
}

impl<B: ?Sized + DynDynBase, P: Deref + Copy> Copy for DynDynFat<B, P> where P::Target: Unsize<B> {}

impl<B: ?Sized + DynDynBase, P: StableDeref + Default> Default for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    fn default() -> Self {
        DynDynFat::new(Default::default())
    }
}

impl<B: ?Sized + DynDynBase, P: StableDeref> From<P> for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    fn from(ptr: P) -> Self {
        DynDynFat::new(ptr)
    }
}

impl<B: ?Sized + DynDynBase, P: Deref> AsRef<P> for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
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

impl<B: ?Sized + DynDynBase, P: Deref + Display> Display for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.ptr)
    }
}

impl<B: ?Sized + DynDynBase, P: Deref + Pointer> Pointer for DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}
