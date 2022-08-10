use crate::{DynDyn, DynDynBase, DynDynTable, StableDynDyn};
use core::marker::{PhantomData, Unsize};
use core::ops::{Deref, DerefMut};
use core::ptr;
use stable_deref_trait::{CloneStableDeref, StableDeref};

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
    pub unsafe fn new_unchecked(ptr: P) -> Self {
        let table = DynDyn::<B>::deref_dyn_dyn(&ptr).1;

        DynDynFat {
            ptr,
            table,
            _base: PhantomData,
        }
    }

    pub fn deref_fat(ptr: &Self) -> DynDynFat<B, &P::Target> {
        DynDynFat {
            ptr: ptr.ptr.deref(),
            table: ptr.table,
            _base: PhantomData,
        }
    }

    pub fn unwrap(ptr: Self) -> P {
        ptr.ptr
    }

    pub fn get_dyn_dyn_table(ptr: &Self) -> DynDynTable {
        ptr.table
    }
}

impl<B: ?Sized + DynDynBase, P: DerefMut> DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    pub fn deref_mut_fat(ptr: &mut Self) -> DynDynFat<B, &mut P::Target> {
        DynDynFat {
            ptr: ptr.ptr.deref_mut(),
            table: ptr.table,
            _base: PhantomData,
        }
    }
}

impl<'a, B: ?Sized + DynDynBase, P: Deref + StableDynDyn<'a, B>> DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
    pub fn new(ptr: P) -> Self {
        unsafe { Self::new_unchecked(ptr) }
    }
}

impl<B: ?Sized + DynDynBase, P: Deref + Clone> DynDynFat<B, P>
where
    P::Target: Unsize<B>,
{
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

unsafe impl<B: ?Sized + DynDynBase, P: StableDeref> StableDeref for DynDynFat<B, P> where
    P::Target: Unsize<B>
{
}
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
            DynDyn::<B>::deref_dyn_dyn(&ptr).1
        };

        DynDynFat {
            ptr,
            table,
            _base: PhantomData,
        }
    }
}

impl<B: ?Sized + DynDynBase, P: Deref + Copy> Copy for DynDynFat<B, P> where P::Target: Unsize<B> {}

#[cfg(all(test, feature = "alloc"))]
mod test {
    use crate::{DynDynBase, DynDynFat, DynDynTable, DynDynTableEntry};
    use alloc::boxed::Box;
    use alloc::rc::Rc;
    use core::cell::Cell;
    use core::ops::Deref;
    use dyn_dyn_macros::dyn_dyn_cast;
    use stable_deref_trait::StableDeref;

    // We need the pointers to these two tables to be distinct in order to properly differentiate them, so these cannot be declared as
    // statics with ZSTs.
    static EMPTY_TABLE_1: (u8, [DynDynTableEntry; 0]) = (0, []);
    static EMPTY_TABLE_2: (u8, [DynDynTableEntry; 0]) = (0, []);

    trait Base {
        fn get_dyn_dyn_table(&self) -> DynDynTable;
    }

    impl<'a> DynDynBase for dyn Base + 'a {
        fn get_dyn_dyn_table(&self) -> DynDynTable {
            self.get_dyn_dyn_table()
        }
    }

    #[derive(Debug, Clone)]
    struct TestStruct<'a>(&'a Cell<usize>, &'static [DynDynTableEntry]);

    impl<'a> Base for TestStruct<'a> {
        fn get_dyn_dyn_table(&self) -> DynDynTable {
            self.0.set(self.0.get() + 1);
            DynDynTable::new(self.1)
        }
    }

    #[derive(Debug)]
    struct WeirdCloneBox<'a>(Box<TestStruct<'a>>);

    impl<'a> Clone for WeirdCloneBox<'a> {
        fn clone(&self) -> Self {
            WeirdCloneBox(Box::new(TestStruct((*self.0).0, &EMPTY_TABLE_2.1)))
        }
    }

    impl<'a> Deref for WeirdCloneBox<'a> {
        type Target = TestStruct<'a>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    unsafe impl<'a> StableDeref for WeirdCloneBox<'a> {}

    #[test]
    fn test_get_table_cached() {
        let num_table_calls = Cell::new(0);
        let mut ptr: DynDynFat<dyn Base, _> =
            DynDynFat::new(Box::new(TestStruct(&num_table_calls, &EMPTY_TABLE_1.1)));

        assert_eq!(1, num_table_calls.get());
        assert_eq!(
            &EMPTY_TABLE_1.1[..] as *const _,
            DynDynFat::get_dyn_dyn_table(&ptr).traits as *const _
        );
        dyn_dyn_cast!(Base => Base, &ptr);
        dyn_dyn_cast!(mut Base => Base, &mut ptr);
        assert_eq!(1, num_table_calls.get());
    }

    #[test]
    fn test_clone_unstable() {
        let num_table_calls = Cell::new(0);
        let ptr: DynDynFat<dyn Base, _> =
            DynDynFat::new(Box::new(TestStruct(&num_table_calls, &EMPTY_TABLE_1.1))).clone();

        assert_eq!(2, num_table_calls.get());
        assert_eq!(
            &EMPTY_TABLE_1.1[..] as *const _,
            DynDynFat::get_dyn_dyn_table(&ptr).traits as *const _
        );
    }

    #[test]
    fn test_clone_stable() {
        let num_table_calls = Cell::new(0);
        let ptr: DynDynFat<dyn Base, _> =
            DynDynFat::new(Rc::new(TestStruct(&num_table_calls, &EMPTY_TABLE_1.1))).clone();

        assert_eq!(1, num_table_calls.get());
        assert_eq!(
            &EMPTY_TABLE_1.1[..] as *const _,
            DynDynFat::get_dyn_dyn_table(&ptr).traits as *const _
        );
    }

    #[test]
    fn test_clone_changes_table() {
        let num_table_calls = Cell::new(0);
        let ptr: DynDynFat<dyn Base, _> = DynDynFat::new(WeirdCloneBox(Box::new(TestStruct(
            &num_table_calls,
            &EMPTY_TABLE_1.1,
        ))))
        .clone();

        assert_eq!(2, num_table_calls.get());
        assert_eq!(
            &EMPTY_TABLE_2.1[..] as *const _,
            DynDynFat::get_dyn_dyn_table(&ptr).traits as *const _
        );
    }
}
