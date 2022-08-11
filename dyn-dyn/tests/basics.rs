use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_derived};
use std::fmt;
use std::rc::Rc;

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

    assert_eq!(Some(1), dyn_dyn_cast!(Base => SubTraitA, d).map(|a| a.a()));
    assert_eq!(Some(2), dyn_dyn_cast!(Base => SubTraitB, d).map(|b| b.b()));
    assert_eq!(None, dyn_dyn_cast!(Base => SubTraitC, d).map(|c| c.c()));
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

    let t = dyn_dyn_cast!(Base => TestTrait, &test);

    assert_eq!(Some(&test as *const _), t.map(|t| t.test()));
}

#[test]
fn test_from_alloc() {
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

    let test_box = Box::new(TestStruct);

    assert_eq!(
        Some(&*test_box as *const _),
        dyn_dyn_cast!(Base => TestTrait, &test_box).map(|t| t.test())
    );

    let test_rc = Rc::new(TestStruct);

    assert_eq!(
        Some(&*test_rc as *const _),
        dyn_dyn_cast!(Base => TestTrait, &test_rc).map(|t| t.test())
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
        dyn_dyn_cast!(Base => fmt::Debug, &TestStruct("abc") as &dyn Base)
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

    assert!(dyn_dyn_cast!(Base => TestTrait, &0_u32 as &dyn Base).is_some());
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

    assert!(dyn_dyn_cast!(BaseA => TraitA, &TestStruct as &dyn BaseA).is_some());
    assert!(dyn_dyn_cast!(BaseA => TraitB, &TestStruct as &dyn BaseA).is_none());

    assert!(dyn_dyn_cast!(BaseB => TraitA, &TestStruct as &dyn BaseB).is_none());
    assert!(dyn_dyn_cast!(BaseB => TraitB, &TestStruct as &dyn BaseB).is_some());
}
