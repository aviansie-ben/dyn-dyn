use std::fmt;
use dyn_dyn::DynDyn;
use dyn_dyn_macros::{dyn_dyn_base, dyn_dyn_derived};

#[dyn_dyn_base]
trait Base {}

trait SubTraitA {
    fn a(&self) -> u32;
}

trait SubTraitB {
    fn b(&self) -> u32;
}

trait SubTraitC {
    fn c(&self) -> u32;
}

struct StructA;

#[dyn_dyn_derived(SubTraitA, SubTraitB)]
impl Base for StructA {}

impl SubTraitA for StructA {
    fn a(&self) -> u32 {
        1
    }
}

impl SubTraitB for StructA {
    fn b(&self) -> u32 {
        2
    }
}

#[test]
fn test_vtable_correct() {
    let d = &StructA as &dyn Base;

    assert_eq!(Some(1), d.try_downcast::<dyn SubTraitA>().map(|a| a.a()));
    assert_eq!(Some(2), d.try_downcast::<dyn SubTraitB>().map(|b| b.b()));
    assert_eq!(None, d.try_downcast::<dyn SubTraitC>().map(|c| c.c()));
}

struct StructB(u32);

#[dyn_dyn_derived(SubTraitC)]
impl Base for StructB {}

impl SubTraitC for StructB {
    fn c(&self) -> u32 {
        self.0
    }
}

#[test]
fn test_data_pointer_correct() {
    assert_eq!(Some(0), (&StructB(0) as &dyn Base).try_downcast::<dyn SubTraitC>().map(|c| c.c()));
    assert_eq!(Some(999), (&StructB(999) as &dyn Base).try_downcast::<dyn SubTraitC>().map(|c| c.c()));
    assert_eq!(Some(9999), (&StructB(9999) as &dyn Base).try_downcast::<dyn SubTraitC>().map(|c| c.c()));
}

struct StructC(&'static str);

#[dyn_dyn_derived(fmt::Debug)]
impl Base for StructC {}

impl fmt::Debug for StructC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StructC({:?})", self.0)
    }
}

#[test]
fn test_external_trait() {
    assert_eq!(Some("StructC(\"abc\")".to_owned()), (&StructC("abc") as &dyn Base).try_downcast::<dyn fmt::Debug>().map(|f| format!("{:?}", f)));
}
