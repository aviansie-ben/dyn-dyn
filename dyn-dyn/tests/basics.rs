use dyn_dyn_macros::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_derived};
use std::fmt;

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

    assert_eq!(Some(1), dyn_dyn_cast!(d as &dyn SubTraitA).map(|a| a.a()));
    assert_eq!(Some(2), dyn_dyn_cast!(d as &dyn SubTraitB).map(|b| b.b()));
    assert_eq!(None, dyn_dyn_cast!(d as &dyn SubTraitC).map(|c| c.c()));
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
        fn test(&self) -> *const TestStruct {
            self
        }
    }

    let test = TestStruct;

    assert_eq!(
        Some(&test as *const _),
        dyn_dyn_cast!((&test as &dyn Base) as &dyn TestTrait).map(|t| t.test())
    );
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

    assert_eq!(
        Some("TestStruct(\"abc\")".to_owned()),
        dyn_dyn_cast!((&TestStruct("abc") as &dyn Base) as &dyn fmt::Debug)
            .map(|f| format!("{:?}", f))
    );
}

#[test]
fn test_external_type() {
    #[dyn_dyn_base]
    trait Base {}
    trait TestTrait {}

    #[dyn_dyn_derived(TestTrait)]
    impl Base for u32 {}
    impl TestTrait for u32 {}

    assert!(dyn_dyn_cast!((&0_u32 as &dyn Base) as &dyn TestTrait).is_some());
}

#[test]
fn test_multi_base() {
    #[dyn_dyn_base]
    trait BaseA {}
    trait TraitA {}

    #[dyn_dyn_base]
    trait BaseB {}
    trait TraitB {}

    struct TestStruct;

    #[dyn_dyn_derived(TraitA)]
    impl BaseA for TestStruct {}
    impl TraitA for TestStruct {}

    #[dyn_dyn_derived(TraitB)]
    impl BaseB for TestStruct {}
    impl TraitB for TestStruct {}

    assert!(dyn_dyn_cast!((&TestStruct as &dyn BaseA) as &dyn TraitA).is_some());
    assert!(dyn_dyn_cast!((&TestStruct as &dyn BaseA) as &dyn TraitB).is_none());

    assert!(dyn_dyn_cast!((&TestStruct as &dyn BaseB) as &dyn TraitA).is_none());
    assert!(dyn_dyn_cast!((&TestStruct as &dyn BaseB) as &dyn TraitB).is_some());
}
