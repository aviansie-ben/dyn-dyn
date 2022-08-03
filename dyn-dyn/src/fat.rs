use crate::{DynDynImpl, DynDynTable};
use stable_deref_trait::{CloneStableDeref, StableDeref};
use std::ops::{Deref, DerefMut};
use std::ptr;

#[derive(Debug)]
pub struct DynDynFat<P: Deref>
where
    P::Target: DynDynImpl,
{
    ptr: P,
    table: DynDynTable,
}

impl<P: Deref> DynDynImpl for DynDynFat<P>
where
    P::Target: DynDynImpl,
{
    type BaseDynDyn = <P::Target as DynDynImpl>::BaseDynDyn;

    const IS_SEND: bool = P::Target::IS_SEND;
    const IS_SYNC: bool = P::Target::IS_SYNC;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        self.table
    }
}

impl<P: Deref> DynDynFat<P>
where
    P::Target: DynDynImpl,
{
    pub unsafe fn new_unchecked(ptr: P) -> DynDynFat<P> {
        let table = ptr.get_dyn_dyn_table();

        DynDynFat { ptr, table }
    }

    pub fn deref_fat(ptr: &Self) -> DynDynFat<&P::Target> {
        DynDynFat {
            ptr: ptr.ptr.deref(),
            table: ptr.table,
        }
    }

    pub fn unwrap(ptr: Self) -> P {
        ptr.ptr
    }
}

impl<P: DerefMut> DynDynFat<P>
where
    P::Target: DynDynImpl,
{
    pub fn deref_mut_fat(ptr: &mut Self) -> DynDynFat<&mut P::Target> {
        DynDynFat {
            ptr: ptr.ptr.deref_mut(),
            table: ptr.table,
        }
    }
}

impl<P: StableDeref> DynDynFat<P>
where
    P::Target: DynDynImpl,
{
    pub fn new(ptr: P) -> DynDynFat<P> {
        unsafe { Self::new_unchecked(ptr) }
    }
}

impl<P: Deref + Clone> DynDynFat<P>
where
    P::Target: DynDynImpl,
{
    pub unsafe fn clone_unchecked(ptr: &Self) -> Self {
        DynDynFat {
            ptr: ptr.ptr.clone(),
            table: ptr.table,
        }
    }
}

impl<P: Deref> Deref for DynDynFat<P>
where
    P::Target: DynDynImpl,
{
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        self.ptr.deref()
    }
}

unsafe impl<P: StableDeref> StableDeref for DynDynFat<P> where P::Target: DynDynImpl {}
unsafe impl<P: CloneStableDeref> CloneStableDeref for DynDynFat<P> where P::Target: DynDynImpl {}

impl<P: DerefMut> DerefMut for DynDynFat<P>
where
    P::Target: DynDynImpl,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.ptr.deref_mut()
    }
}

impl<P: Deref + Clone> Clone for DynDynFat<P>
where
    P::Target: DynDynImpl,
{
    fn clone(&self) -> Self {
        let ptr = self.ptr.clone();
        let table = if ptr::eq(ptr.deref(), self.ptr.deref()) {
            self.table
        } else {
            ptr.get_dyn_dyn_table()
        };

        DynDynFat { ptr, table }
    }
}

impl<P: Deref + Copy> Copy for DynDynFat<P> where P::Target: DynDynImpl {}

#[cfg(test)]
mod test {
    use crate::{DynDynFat, DynDynImpl, DynDynTable, DynDynTableEntry};
    use stable_deref_trait::StableDeref;
    use std::cell::Cell;
    use std::ops::Deref;
    use std::rc::Rc;

    // We need the pointers to these two tables to be distinct in order to properly differentiate them, so these cannot be declared as
    // statics with ZSTs.
    static EMPTY_TABLE_1: (u8, [DynDynTableEntry; 0]) = (0, []);
    static EMPTY_TABLE_2: (u8, [DynDynTableEntry; 0]) = (0, []);

    #[derive(Debug, Clone)]
    struct TestStruct<'a>(&'a Cell<usize>, &'static [DynDynTableEntry]);

    impl<'a> DynDynImpl for TestStruct<'a> {
        type BaseDynDyn = TestStruct<'a>;
        const IS_SEND: bool = false;
        const IS_SYNC: bool = false;

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
            &*self.0
        }
    }

    unsafe impl<'a> StableDeref for WeirdCloneBox<'a> {}

    #[test]
    fn test_get_table_cached() {
        let num_table_calls = Cell::new(0);
        let ptr = DynDynFat::new(Box::new(TestStruct(&num_table_calls, &EMPTY_TABLE_1.1)));

        assert_eq!(1, num_table_calls.get());
        assert_eq!(
            &EMPTY_TABLE_1.1[..] as *const _,
            ptr.get_dyn_dyn_table().traits as *const _
        );
        assert_eq!(1, num_table_calls.get());
    }

    #[test]
    fn test_clone_unstable() {
        let num_table_calls = Cell::new(0);
        let ptr = DynDynFat::new(Box::new(TestStruct(&num_table_calls, &EMPTY_TABLE_1.1))).clone();

        assert_eq!(2, num_table_calls.get());
        assert_eq!(
            &EMPTY_TABLE_1.1[..] as *const _,
            ptr.get_dyn_dyn_table().traits as *const _
        );
    }

    #[test]
    fn test_clone_stable() {
        let num_table_calls = Cell::new(0);
        let ptr = DynDynFat::new(Rc::new(TestStruct(&num_table_calls, &EMPTY_TABLE_1.1))).clone();

        assert_eq!(1, num_table_calls.get());
        assert_eq!(
            &EMPTY_TABLE_1.1[..] as *const _,
            ptr.get_dyn_dyn_table().traits as *const _
        );
    }

    #[test]
    fn test_clone_changes_table() {
        let num_table_calls = Cell::new(0);
        let ptr = DynDynFat::new(WeirdCloneBox(Box::new(TestStruct(
            &num_table_calls,
            &EMPTY_TABLE_1.1,
        ))))
        .clone();

        assert_eq!(2, num_table_calls.get());
        assert_eq!(
            &EMPTY_TABLE_2.1[..] as *const _,
            ptr.get_dyn_dyn_table().traits as *const _
        );
    }
}
