use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_impl};

#[test]
fn test_generic_base() {
    #[dyn_dyn_base]
    trait Base<T: 'static> {}

    trait TestTraitA {}
    trait TestTraitB {}

    struct TestStruct;

    #[dyn_dyn_impl(TestTraitA)]
    impl Base<u32> for TestStruct {}

    #[dyn_dyn_impl(TestTraitB)]
    impl Base<u64> for TestStruct {}

    impl TestTraitA for TestStruct {}
    impl TestTraitB for TestStruct {}

    assert!(dyn_dyn_cast!(Base<u32> => TestTraitA, &TestStruct as &dyn Base<u32>).is_ok());
    assert!(dyn_dyn_cast!(Base<u32> => TestTraitB, &TestStruct as &dyn Base<u32>).is_err());

    assert!(dyn_dyn_cast!(Base<u64> => TestTraitA, &TestStruct as &dyn Base<u64>).is_err());
    assert!(dyn_dyn_cast!(Base<u64> => TestTraitB, &TestStruct as &dyn Base<u64>).is_ok());
}

#[test]
fn test_generic_trait() {
    #[dyn_dyn_base]
    trait Base {}

    trait GenericTrait<T: 'static> {
        fn test(&self) -> u32;
    }

    struct TestStruct;

    #[dyn_dyn_impl(GenericTrait<u32>, GenericTrait<u64>)]
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
        Ok(0),
        dyn_dyn_cast!(Base => GenericTrait<u32>, &TestStruct as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(1),
        dyn_dyn_cast!(Base => GenericTrait<u64>, &TestStruct as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Err(()),
        dyn_dyn_cast!(Base => GenericTrait<u16>, &TestStruct as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
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

    #[dyn_dyn_impl(GenericTrait<T>)]
    impl<T: 'static> Base for TestStruct<T> {}

    impl<T: 'static> GenericTrait<T> for TestStruct<T> {
        fn test(&self) -> u32 {
            1234
        }
    }

    assert_eq!(
        Ok(1234),
        dyn_dyn_cast!(Base => GenericTrait<u32>, &TestStruct(0_u32) as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Err(()),
        dyn_dyn_cast!(Base => GenericTrait<u64>, &TestStruct(0_u32) as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );

    assert_eq!(
        Err(()),
        dyn_dyn_cast!(Base => GenericTrait<u32>, &TestStruct(0_u64) as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(1234),
        dyn_dyn_cast!(Base => GenericTrait<u64>, &TestStruct(0_u64) as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );
}

#[test]
fn test_generic_base_from_param() {
    #[dyn_dyn_base]
    trait Base<T: 'static> {}
    trait TestTrait<T: 'static> {}

    struct TestStruct<T: 'static>(T);

    #[dyn_dyn_impl(TestTrait<T>)]
    impl<T: 'static> Base<T> for TestStruct<T> {}
    impl<T: 'static> TestTrait<T> for TestStruct<T> {}

    assert!(
        dyn_dyn_cast!(Base<u32> => TestTrait<u32>, &TestStruct(0_u32) as &dyn Base<u32>).is_ok()
    );
    assert!(
        dyn_dyn_cast!(Base<u32> => TestTrait<u64>, &TestStruct(0_u32) as &dyn Base<u32>).is_err()
    );
    assert!(
        dyn_dyn_cast!(Base<u64> => TestTrait<u32>, &TestStruct(0_u64) as &dyn Base<u64>).is_err()
    );
    assert!(
        dyn_dyn_cast!(Base<u64> => TestTrait<u64>, &TestStruct(0_u64) as &dyn Base<u64>).is_ok()
    );
}

#[test]
fn test_generic_base_blanket_impl() {
    #[dyn_dyn_base]
    trait Base<T: 'static> {}
    trait TestTrait<T: 'static> {}

    struct TestStruct;

    #[dyn_dyn_impl(TestTrait<T>)]
    impl<T: 'static> Base<T> for TestStruct {}
    impl<T: 'static> TestTrait<T> for TestStruct {}

    assert!(dyn_dyn_cast!(Base<u32> => TestTrait<u32>, &TestStruct as &dyn Base<u32>).is_ok());
    assert!(dyn_dyn_cast!(Base<u32> => TestTrait<u64>, &TestStruct as &dyn Base<u32>).is_err());
    assert!(dyn_dyn_cast!(Base<u64> => TestTrait<u32>, &TestStruct as &dyn Base<u64>).is_err());
    assert!(dyn_dyn_cast!(Base<u64> => TestTrait<u64>, &TestStruct as &dyn Base<u64>).is_ok());
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

    #[dyn_dyn_impl(TestTrait)]
    impl Base<u32> for TestStruct {}
    impl TestTrait for TestStruct {}

    assert!(dyn_dyn_cast!(Base<u32> => TestTrait, &TestStruct as &dyn Base<u32>).is_ok());
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

    #[dyn_dyn_impl(GenericTrait<T>)]
    impl<T> Base for TestStruct<T> where T: 'static {}

    impl<T: 'static> GenericTrait<T> for TestStruct<T> {
        fn test(&self) -> u32 {
            1234
        }
    }

    assert_eq!(
        Ok(1234),
        dyn_dyn_cast!(Base => GenericTrait<u32>, &TestStruct(0_u32) as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Err(()),
        dyn_dyn_cast!(Base => GenericTrait<u64>, &TestStruct(0_u32) as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );

    assert_eq!(
        Err(()),
        dyn_dyn_cast!(Base => GenericTrait<u32>, &TestStruct(0_u64) as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(1234),
        dyn_dyn_cast!(Base => GenericTrait<u64>, &TestStruct(0_u64) as &dyn Base)
            .map(|b| b.test())
            .map_err(|_| ())
    );
}
