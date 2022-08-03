use std::fmt;
use dyn_dyn::DynDyn;
use dyn_dyn_macros::{dyn_dyn_base, dyn_dyn_derived};

#[test]
fn test_vtable_correct() {
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

    struct TestStruct;

    #[dyn_dyn_derived(SubTraitA, SubTraitB)]
    impl Base for TestStruct {}

    impl SubTraitA for TestStruct {
        fn a(&self) -> u32 {
            1
        }
    }

    impl SubTraitB for TestStruct {
        fn b(&self) -> u32 {
            2
        }
    }

    let d = &TestStruct as &dyn Base;

    assert_eq!(Some(1), d.try_downcast::<dyn SubTraitA>().map(|a| a.a()));
    assert_eq!(Some(2), d.try_downcast::<dyn SubTraitB>().map(|b| b.b()));
    assert_eq!(None, d.try_downcast::<dyn SubTraitC>().map(|c| c.c()));
}

#[test]
fn test_data_pointer_correct() {
    #[dyn_dyn_base]
    trait Base {}
    trait TestTrait {
        fn test(&self) -> *const TestStruct;
    }

    struct TestStruct;

    #[dyn_dyn_derived(TestTrait)]
    impl Base for TestStruct {}

    impl TestTrait for TestStruct {
        fn test(&self) -> *const TestStruct { self }
    }

    let test = TestStruct;

    assert_eq!(Some(&test as *const _), (&test as &dyn Base).try_downcast::<dyn TestTrait>().map(|t| t.test()));
}

#[test]
fn test_external_trait() {
    #[dyn_dyn_base]
    trait Base {}

    struct TestStruct(&'static str);

    #[dyn_dyn_derived(fmt::Debug)]
    impl Base for TestStruct {}

    impl fmt::Debug for TestStruct {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "TestStruct({:?})", self.0)
        }
    }

    assert_eq!(Some("TestStruct(\"abc\")".to_owned()), (&TestStruct("abc") as &dyn Base).try_downcast::<dyn fmt::Debug>().map(|f| format!("{:?}", f)));
}

#[test]
fn test_external_type() {
    #[dyn_dyn_base]
    trait Base {}
    trait TestTrait {}

    #[dyn_dyn_derived(TestTrait)]
    impl Base for u32 {}
    impl TestTrait for u32 {}

    assert!((&0_u32 as &dyn Base).try_downcast::<dyn TestTrait>().is_some());
}
