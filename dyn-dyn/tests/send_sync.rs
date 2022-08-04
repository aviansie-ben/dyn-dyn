use dyn_dyn_macros::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_derived};

#[dyn_dyn_base]
trait BaseTrait {}
trait TestTrait {
    fn test(&self) -> u32;
}

struct TestStruct;

#[dyn_dyn_derived(TestTrait)]
impl BaseTrait for TestStruct {}
impl TestTrait for TestStruct {
    fn test(&self) -> u32 {
        0xdeadbeef
    }
}

#[test]
fn test_plain_dyn_from_send_sync() {
    assert_eq!(
        Some(0xdeadbeef),
        dyn_dyn_cast!((&TestStruct as &(dyn BaseTrait + Send)) as &dyn TestTrait).map(|t| t.test())
    );
    assert_eq!(
        Some(0xdeadbeef),
        dyn_dyn_cast!((&TestStruct as &(dyn BaseTrait + Sync)) as &dyn TestTrait).map(|t| t.test())
    );
    assert_eq!(
        Some(0xdeadbeef),
        dyn_dyn_cast!((&TestStruct as &(dyn BaseTrait + Send + Sync)) as &dyn TestTrait)
            .map(|t| t.test())
    );
}

#[test]
fn test_send_dyn() {
    assert_eq!(
        Some(0xdeadbeef),
        dyn_dyn_cast!((&TestStruct as &(dyn BaseTrait + Send)) as &(dyn TestTrait + Send))
            .map(|t| t.test())
    );
    assert_eq!(
        Some(0xdeadbeef),
        dyn_dyn_cast!((&TestStruct as &(dyn BaseTrait + Send + Sync)) as &(dyn TestTrait + Send))
            .map(|t| t.test())
    );
}

#[test]
fn test_sync_dyn() {
    assert_eq!(
        Some(0xdeadbeef),
        dyn_dyn_cast!((&TestStruct as &(dyn BaseTrait + Sync)) as &(dyn TestTrait + Sync))
            .map(|t| t.test())
    );
    assert_eq!(
        Some(0xdeadbeef),
        dyn_dyn_cast!((&TestStruct as &(dyn BaseTrait + Send + Sync)) as &(dyn TestTrait + Sync))
            .map(|t| t.test())
    );
}

#[test]
fn test_send_sync_dyn() {
    assert_eq!(
        Some(0xdeadbeef),
        dyn_dyn_cast!(
            (&TestStruct as &(dyn BaseTrait + Send + Sync)) as &(dyn TestTrait + Send + Sync)
        )
        .map(|t| t.test())
    );
}
