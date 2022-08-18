#![cfg(feature = "alloc")]

extern crate alloc;

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::Cell;
use core::ops::Deref;
use dyn_dyn::{dyn_dyn_cast, DynDynBase, DynDynFat, DynDynTable, DynDynTableEntry, GetDynDynTable};
use stable_deref_trait::StableDeref;

// We need the pointers to these two tables to be distinct in order to properly differentiate them, so these cannot be declared as
// statics with ZSTs.
static EMPTY_TABLE_1: (u8, [DynDynTableEntry; 0]) = (0, []);
static EMPTY_TABLE_2: (u8, [DynDynTableEntry; 0]) = (0, []);

trait Base {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

// SAFETY: This is for testing purposes only, so we rely on internal implementation details that get around the safety requirements
unsafe impl<'a> DynDynBase for dyn Base + 'a {
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

// SAFETY: Just a wrapper around Box<TestStruct<'a>>
unsafe impl<'a> StableDeref for WeirdCloneBox<'a> {}

unsafe impl<'a> GetDynDynTable<dyn Base + 'a> for WeirdCloneBox<'a> {
    type DynTarget = dyn Base;

    fn get_dyn_dyn_table(&self) -> DynDynTable {
        <&TestStruct as GetDynDynTable<dyn Base + 'a>>::get_dyn_dyn_table(&&*self.0)
    }
}

#[test]
fn test_get_table_cached() {
    let num_table_calls = Cell::new(0);
    let mut ptr: DynDynFat<dyn Base, _> =
        DynDynFat::new(Box::new(TestStruct(&num_table_calls, &EMPTY_TABLE_1.1)));

    assert_eq!(1, num_table_calls.get());
    assert_eq!(
        &EMPTY_TABLE_1.1[..] as *const _,
        DynDynFat::get_dyn_dyn_table(&ptr).into_slice() as *const _
    );
    let _ = dyn_dyn_cast!(Base => Base, &ptr);
    let _ = dyn_dyn_cast!(mut Base => Base, &mut ptr);
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
        DynDynFat::get_dyn_dyn_table(&ptr).into_slice() as *const _
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
        DynDynFat::get_dyn_dyn_table(&ptr).into_slice() as *const _
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
        DynDynFat::get_dyn_dyn_table(&ptr).into_slice() as *const _
    );
}
