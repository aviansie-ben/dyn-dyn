use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_impl};
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

    #[dyn_dyn_impl(SubTraitA, SubTraitB)]
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

    assert_eq!(
        Ok(1),
        dyn_dyn_cast!(Base => SubTraitA, d)
            .map(|a: &dyn SubTraitA| a.a())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(2),
        dyn_dyn_cast!(Base => SubTraitB, d)
            .map(|b: &dyn SubTraitB| b.b())
            .map_err(|_| ())
    );
    assert_eq!(
        Err(()),
        dyn_dyn_cast!(Base => SubTraitC, d)
            .map(|c: &dyn SubTraitC| c.c())
            .map_err(|_| ())
    );

    let d = &mut TestStruct as &mut dyn Base;

    assert_eq!(
        Ok(1),
        dyn_dyn_cast!(mut Base => SubTraitA, d)
            .map(|a: &mut dyn SubTraitA| a.a())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(2),
        dyn_dyn_cast!(mut Base => SubTraitB, d)
            .map(|b: &mut dyn SubTraitB| b.b())
            .map_err(|_| ())
    );
    assert_eq!(
        Err(()),
        dyn_dyn_cast!(mut Base => SubTraitC, d)
            .map(|c: &mut dyn SubTraitC| c.c())
            .map_err(|_| ())
    );
}

#[test]
fn test_data_pointer_correct() {
    #[dyn_dyn_base]
    trait Base {}
    trait TestTrait {
        fn test(&self) -> *const TestStruct;
        fn test_mut(&mut self) -> *mut TestStruct;
    }

    struct TestStruct;

    #[dyn_dyn_impl(TestTrait)]
    impl Base for TestStruct {}

    impl TestTrait for TestStruct {
        fn test(&self) -> *const TestStruct {
            self
        }

        fn test_mut(&mut self) -> *mut TestStruct {
            self
        }
    }

    let mut test = TestStruct;

    assert_eq!(
        Ok(&test as *const _),
        dyn_dyn_cast!(Base => TestTrait, &test)
            .map(|t| t.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(&test as *const _ as *mut _),
        dyn_dyn_cast!(mut Base => TestTrait, &mut test)
            .map(|t| t.test_mut())
            .map_err(|_| ())
    );
}

#[cfg(feature = "alloc")]
#[test]
fn test_from_alloc() {
    use std::rc::Rc;

    #[dyn_dyn_base]
    trait Base {}
    trait TestTrait {
        fn test(&self) -> *const TestStruct;
    }

    struct TestStruct;

    #[dyn_dyn_impl(TestTrait)]
    impl Base for TestStruct {}

    impl TestTrait for TestStruct {
        fn test(&self) -> *const TestStruct {
            self
        }
    }

    let mut test_box = Box::new(TestStruct);
    let box_ptr = &*test_box as *const _;

    assert_eq!(
        Ok(box_ptr),
        dyn_dyn_cast!(Base => TestTrait, &test_box)
            .map(|t: &dyn TestTrait| t.test())
            .map_err(|_| ())
    );

    assert_eq!(
        Ok(box_ptr),
        dyn_dyn_cast!(mut Base => TestTrait, &mut test_box)
            .map(|t: &mut dyn TestTrait| t.test())
            .map_err(|_| ())
    );

    assert_eq!(
        Ok(box_ptr),
        dyn_dyn_cast!(move Base => TestTrait, test_box)
            .map(|t: Box<dyn TestTrait>| t.test())
            .map_err(|_| ())
    );

    let test_rc = Rc::new(TestStruct);
    let rc_ptr = &*test_rc as *const _;

    assert_eq!(
        Ok(rc_ptr),
        dyn_dyn_cast!(Base => TestTrait, &test_rc)
            .map(|t| t.test())
            .map_err(|_| ())
    );

    assert_eq!(
        Ok(rc_ptr),
        dyn_dyn_cast!(move Base => TestTrait, test_rc)
            .map(|t| t.test())
            .map_err(|_| ())
    );
}

#[test]
fn test_external_trait() {
    #[dyn_dyn_base]
    trait Base {}

    struct TestStruct(&'static str);

    #[dyn_dyn_impl(fmt::Debug)]
    impl Base for TestStruct {}

    impl fmt::Debug for TestStruct {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "TestStruct({:?})", self.0)
        }
    }

    assert_eq!(
        Ok("TestStruct(\"abc\")".to_owned()),
        dyn_dyn_cast!(Base => fmt::Debug, &TestStruct("abc") as &dyn Base)
            .map(|f| format!("{:?}", f))
            .map_err(|_| ())
    );
}

mod test_external_type {
    use super::*;

    #[dyn_dyn_base]
    trait Base {}
    trait TestTrait {}

    #[dyn_dyn_impl(TestTrait)]
    impl Base for u32 {}
    impl TestTrait for u32 {}

    #[test]
    fn test_external_type() {
        assert!(dyn_dyn_cast!(Base => TestTrait, &0_u32 as &dyn Base).is_ok());
    }
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

    #[dyn_dyn_impl(TraitA)]
    impl BaseA for TestStruct {}
    impl TraitA for TestStruct {}

    #[dyn_dyn_impl(TraitB)]
    impl BaseB for TestStruct {}
    impl TraitB for TestStruct {}

    assert!(dyn_dyn_cast!(BaseA => TraitA, &TestStruct as &dyn BaseA).is_ok());
    assert!(dyn_dyn_cast!(BaseA => TraitB, &TestStruct as &dyn BaseA).is_err());

    assert!(dyn_dyn_cast!(BaseB => TraitA, &TestStruct as &dyn BaseB).is_err());
    assert!(dyn_dyn_cast!(BaseB => TraitB, &TestStruct as &dyn BaseB).is_ok());
}

#[test]
fn test_temporaries_extended() {
    #[dyn_dyn_base]
    trait Base {}
    trait Trait {}

    struct TestStruct;

    #[dyn_dyn_impl(Trait)]
    impl Base for TestStruct {}
    impl Trait for TestStruct {}

    fn return_temporary() -> TestStruct {
        TestStruct
    }

    assert!(dyn_dyn_cast!(Base => Trait, &return_temporary() as &dyn Base).is_ok());
    assert!(dyn_dyn_cast!(mut Base => Trait, &mut return_temporary() as &mut dyn Base).is_ok());
}

#[cfg(feature = "alloc")]
#[test]
fn test_static_lifetime_on_base() {
    #[dyn_dyn_base]
    trait Base: 'static {}
    trait Trait {}

    struct TestStruct;

    #[dyn_dyn_impl(Trait)]
    impl Base for TestStruct {}
    impl Trait for TestStruct {}

    let mut boxed = Box::new(TestStruct);

    assert!(dyn_dyn_cast!(Base => Trait, &boxed).is_ok());
    assert!(dyn_dyn_cast!(mut Base => Trait, &mut boxed).is_ok());
    assert!(dyn_dyn_cast!(move Base => Trait, boxed).is_ok());
}
