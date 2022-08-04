use dyn_dyn_macros::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_derived};

#[test]
fn test_generic_base() {
    #[dyn_dyn_base]
    trait Base<T: 'static> {}

    trait TestTraitA {}
    trait TestTraitB {}

    struct TestStruct;

    #[dyn_dyn_derived(TestTraitA)]
    impl Base<u32> for TestStruct {}

    #[dyn_dyn_derived(TestTraitB)]
    impl Base<u64> for TestStruct {}

    impl TestTraitA for TestStruct {}
    impl TestTraitB for TestStruct {}

    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u32>) as &dyn TestTraitA).is_some());
    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u32>) as &dyn TestTraitB).is_none());

    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u64>) as &dyn TestTraitA).is_none());
    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u64>) as &dyn TestTraitB).is_some());
}

#[test]
fn test_generic_trait() {
    #[dyn_dyn_base]
    trait Base {}

    trait GenericTrait<T: 'static> {
        fn test(&self) -> u32;
    }

    struct TestStruct;

    #[dyn_dyn_derived(GenericTrait<u32>, GenericTrait<u64>)]
    impl Base for TestStruct {}
    impl GenericTrait<u32> for TestStruct {
        fn test(&self) -> u32 {
            0
        }
    }
    impl GenericTrait<u64> for TestStruct {
        fn test(&self) -> u32 {
            1
        }
    }

    assert_eq!(
        Some(0),
        dyn_dyn_cast!((&TestStruct as &dyn Base) as &dyn GenericTrait<u32>).map(|b| b.test())
    );
    assert_eq!(
        Some(1),
        dyn_dyn_cast!((&TestStruct as &dyn Base) as &dyn GenericTrait<u64>).map(|b| b.test())
    );
    assert_eq!(
        None,
        dyn_dyn_cast!((&TestStruct as &dyn Base) as &dyn GenericTrait<u16>).map(|b| b.test())
    );
}

#[test]
fn test_generic_trait_from_param() {
    #[dyn_dyn_base]
    trait Base {}

    trait GenericTrait<T: 'static> {
        fn test(&self) -> u32;
    }

    struct TestStruct<T: 'static>(T);

    #[dyn_dyn_derived(GenericTrait<T>)]
    impl<T: 'static> Base for TestStruct<T> {}

    impl<T: 'static> GenericTrait<T> for TestStruct<T> {
        fn test(&self) -> u32 {
            1234
        }
    }

    assert_eq!(
        Some(1234),
        dyn_dyn_cast!((&TestStruct(0_u32) as &dyn Base) as &dyn GenericTrait<u32>)
            .map(|b| b.test())
    );
    assert_eq!(
        None,
        dyn_dyn_cast!((&TestStruct(0_u32) as &dyn Base) as &dyn GenericTrait<u64>)
            .map(|b| b.test())
    );

    assert_eq!(
        None,
        dyn_dyn_cast!((&TestStruct(0_u64) as &dyn Base) as &dyn GenericTrait<u32>)
            .map(|b| b.test())
    );
    assert_eq!(
        Some(1234),
        dyn_dyn_cast!((&TestStruct(0_u64) as &dyn Base) as &dyn GenericTrait<u64>)
            .map(|b| b.test())
    );
}

#[test]
fn test_generic_base_from_param() {
    #[dyn_dyn_base]
    trait Base<T: 'static> {}
    trait TestTrait<T: 'static> {}

    struct TestStruct<T: 'static>(T);

    #[dyn_dyn_derived(TestTrait<T>)]
    impl<T: 'static> Base<T> for TestStruct<T> {}
    impl<T: 'static> TestTrait<T> for TestStruct<T> {}

    assert!(dyn_dyn_cast!((&TestStruct(0_u32) as &dyn Base<u32>) as &dyn TestTrait<u32>).is_some());
    assert!(dyn_dyn_cast!((&TestStruct(0_u32) as &dyn Base<u32>) as &dyn TestTrait<u64>).is_none());
    assert!(dyn_dyn_cast!((&TestStruct(0_u64) as &dyn Base<u64>) as &dyn TestTrait<u32>).is_none());
    assert!(dyn_dyn_cast!((&TestStruct(0_u64) as &dyn Base<u64>) as &dyn TestTrait<u64>).is_some());
}

#[test]
fn test_generic_base_blanket_impl() {
    #[dyn_dyn_base]
    trait Base<T: 'static> {}
    trait TestTrait<T: 'static> {}

    struct TestStruct;

    #[dyn_dyn_derived(TestTrait<T>)]
    impl<T: 'static> Base<T> for TestStruct {}
    impl<T: 'static> TestTrait<T> for TestStruct {}

    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u32>) as &dyn TestTrait<u32>).is_some());
    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u32>) as &dyn TestTrait<u64>).is_none());
    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u64>) as &dyn TestTrait<u32>).is_none());
    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u64>) as &dyn TestTrait<u64>).is_some());
}

#[test]
fn test_where_clause_on_base() {
    #[dyn_dyn_base]
    trait Base<T>
    where
        T: 'static,
    {
    }
    trait TestTrait {}

    struct TestStruct;

    #[dyn_dyn_derived(TestTrait)]
    impl Base<u32> for TestStruct {}
    impl TestTrait for TestStruct {}

    assert!(dyn_dyn_cast!((&TestStruct as &dyn Base<u32>) as &dyn TestTrait).is_some());
}

#[test]
fn test_where_clause_on_derived() {
    #[dyn_dyn_base]
    trait Base {}

    trait GenericTrait<T: 'static> {
        fn test(&self) -> u32;
    }

    struct TestStruct<T>(T)
    where
        T: 'static;

    #[dyn_dyn_derived(GenericTrait<T>)]
    impl<T> Base for TestStruct<T> where T: 'static {}

    impl<T: 'static> GenericTrait<T> for TestStruct<T> {
        fn test(&self) -> u32 {
            1234
        }
    }

    assert_eq!(
        Some(1234),
        dyn_dyn_cast!((&TestStruct(0_u32) as &dyn Base) as &dyn GenericTrait<u32>)
            .map(|b| b.test())
    );
    assert_eq!(
        None,
        dyn_dyn_cast!((&TestStruct(0_u32) as &dyn Base) as &dyn GenericTrait<u64>)
            .map(|b| b.test())
    );

    assert_eq!(
        None,
        dyn_dyn_cast!((&TestStruct(0_u64) as &dyn Base) as &dyn GenericTrait<u32>)
            .map(|b| b.test())
    );
    assert_eq!(
        Some(1234),
        dyn_dyn_cast!((&TestStruct(0_u64) as &dyn Base) as &dyn GenericTrait<u64>)
            .map(|b| b.test())
    );
}
