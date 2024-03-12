#![cfg(feature = "alloc")]

use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_impl};

extern crate alloc;

#[dyn_dyn_base]
trait Base {}

trait Trait {
    fn test(&self) -> &'static str;
}

#[test]
fn test_macro_multi_call() {
    macro_rules! impl_inside_macro {
        ($base:ident, $_trait:ident, $derived:ident) => {
            #[dyn_dyn_impl($_trait)]
            impl $base for $derived {}
        };
    }

    struct S1;
    impl Trait for S1 {
        fn test(&self) -> &'static str {
            "S1"
        }
    }
    impl_inside_macro!(Base, Trait, S1);

    struct S2;
    impl Trait for S2 {
        fn test(&self) -> &'static str {
            "S2"
        }
    }
    impl_inside_macro!(Base, Trait, S2);

    assert_eq!(
        Some("S1"),
        dyn_dyn_cast!(Base => Trait, &S1).map(|t| t.test()).ok()
    );

    assert_eq!(
        Some("S2"),
        dyn_dyn_cast!(Base => Trait, &S2).map(|t| t.test()).ok()
    );
}

#[test]
fn test_macro_multi_impl() {
    macro_rules! impl_inside_macro {
        ($base:ident, $_trait:ident, $derived1:ident, $derived2:ident) => {
            #[dyn_dyn_impl($_trait)]
            impl $base for $derived1 {}

            #[dyn_dyn_impl($_trait)]
            impl $base for $derived2 {}
        };
    }

    struct S1;
    impl Trait for S1 {
        fn test(&self) -> &'static str {
            "S1"
        }
    }

    struct S2;
    impl Trait for S2 {
        fn test(&self) -> &'static str {
            "S2"
        }
    }

    impl_inside_macro!(Base, Trait, S1, S2);

    assert_eq!(
        Some("S1"),
        dyn_dyn_cast!(Base => Trait, &S1).map(|t| t.test()).ok()
    );

    assert_eq!(
        Some("S2"),
        dyn_dyn_cast!(Base => Trait, &S2).map(|t| t.test()).ok()
    );
}
