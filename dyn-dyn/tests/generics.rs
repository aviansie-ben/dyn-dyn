use dyn_dyn::DynDyn;
use dyn_dyn_macros::{dyn_dyn_base, dyn_dyn_derived};

#[dyn_dyn_base]
trait PlainBase {}

#[dyn_dyn_base]
trait GenericBase<T: 'static> {}

trait PlainTrait {}

trait GenericTrait<T: 'static> {
    fn test(&self) -> u32;
}

struct StructA;

#[dyn_dyn_derived(PlainTrait)]
impl GenericBase<u32> for StructA {}
impl PlainTrait for StructA {}

#[test]
fn test_struct_a() {
    assert!((&StructA as &dyn GenericBase<u32>).try_downcast::<dyn PlainTrait>().is_some());
    assert!((&StructA as &dyn GenericBase<u32>).try_downcast::<dyn GenericTrait<u32>>().is_none());
}

struct StructB;

#[dyn_dyn_derived(GenericTrait<u32>, GenericTrait<u64>)]
impl PlainBase for StructB {}
impl GenericTrait<u32> for StructB {
    fn test(&self) -> u32 { 0 }
}
impl GenericTrait<u64> for StructB {
    fn test(&self) -> u32 { 1 }
}

#[test]
fn test_struct_b() {
    assert_eq!(Some(0), (&StructB as &dyn PlainBase).try_downcast::<dyn GenericTrait<u32>>().map(|b| b.test()));
    assert_eq!(Some(1), (&StructB as &dyn PlainBase).try_downcast::<dyn GenericTrait<u64>>().map(|b| b.test()));
    assert_eq!(None, (&StructB as &dyn PlainBase).try_downcast::<dyn GenericTrait<u16>>().map(|b| b.test()));
}

struct StructC<T: 'static>(T);

#[dyn_dyn_derived(GenericTrait<T>)]
impl<T: 'static> PlainBase for StructC<T> {}

impl<T: 'static> GenericTrait<T> for StructC<T> {
    fn test(&self) -> u32 {
        1234
    }
}

#[test]
fn test_struct_c() {
    assert_eq!(Some(1234), (&StructC(0_u32) as &dyn PlainBase).try_downcast::<dyn GenericTrait<u32>>().map(|b| b.test()));
    assert_eq!(None, (&StructC(0_u32) as &dyn PlainBase).try_downcast::<dyn GenericTrait<u64>>().map(|b| b.test()));

    assert_eq!(None, (&StructC(0_u64) as &dyn PlainBase).try_downcast::<dyn GenericTrait<u32>>().map(|b| b.test()));
    assert_eq!(Some(1234), (&StructC(0_u64) as &dyn PlainBase).try_downcast::<dyn GenericTrait<u64>>().map(|b| b.test()));
}

struct StructD<T>(T) where T: 'static;

#[dyn_dyn_derived(GenericTrait<T>)]
impl<T> PlainBase for StructD<T> where T: 'static {}

impl<T: 'static> GenericTrait<T> for StructD<T> {
    fn test(&self) -> u32 {
        1234
    }
}

#[test]
fn test_struct_d() {
    assert_eq!(Some(1234), (&StructD(0_u32) as &dyn PlainBase).try_downcast::<dyn GenericTrait<u32>>().map(|b| b.test()));
    assert_eq!(None, (&StructD(0_u32) as &dyn PlainBase).try_downcast::<dyn GenericTrait<u64>>().map(|b| b.test()));

    assert_eq!(None, (&StructD(0_u64) as &dyn PlainBase).try_downcast::<dyn GenericTrait<u32>>().map(|b| b.test()));
    assert_eq!(Some(1234), (&StructD(0_u64) as &dyn PlainBase).try_downcast::<dyn GenericTrait<u64>>().map(|b| b.test()));
}
