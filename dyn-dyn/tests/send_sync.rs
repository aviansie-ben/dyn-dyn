use dyn_dyn::DynDyn;
use dyn_dyn_macros::{dyn_dyn_base, dyn_dyn_derived};

#[dyn_dyn_base]
trait BaseTrait {}
trait TestTrait {}

struct TestStruct;

#[dyn_dyn_derived(TestTrait)]
impl BaseTrait for TestStruct {}
impl TestTrait for TestStruct {}

#[test]
fn test_plain_dyn_from_send_sync() {
    assert!((&TestStruct as &(dyn BaseTrait + Send)).try_downcast::<dyn TestTrait>().is_some());
    assert!((&TestStruct as &(dyn BaseTrait + Sync)).try_downcast::<dyn TestTrait>().is_some());
    assert!((&TestStruct as &(dyn BaseTrait + Send + Sync)).try_downcast::<dyn TestTrait>().is_some());
}

#[test]
fn test_send_dyn() {
    assert!((&TestStruct as &dyn BaseTrait).try_downcast::<dyn TestTrait + Send>().is_none());
    assert!((&TestStruct as &(dyn BaseTrait + Send)).try_downcast::<dyn TestTrait + Send>().is_some());
    assert!((&TestStruct as &(dyn BaseTrait + Sync)).try_downcast::<dyn TestTrait + Send>().is_none());
    assert!((&TestStruct as &(dyn BaseTrait + Send + Sync)).try_downcast::<dyn TestTrait + Send>().is_some());
}

#[test]
fn test_sync_dyn() {
    assert!((&TestStruct as &dyn BaseTrait).try_downcast::<dyn TestTrait + Sync>().is_none());
    assert!((&TestStruct as &(dyn BaseTrait + Send)).try_downcast::<dyn TestTrait + Sync>().is_none());
    assert!((&TestStruct as &(dyn BaseTrait + Sync)).try_downcast::<dyn TestTrait + Sync>().is_some());
    assert!((&TestStruct as &(dyn BaseTrait + Send + Sync)).try_downcast::<dyn TestTrait + Sync>().is_some());
}

#[test]
fn test_send_sync_dyn() {
    assert!((&TestStruct as &dyn BaseTrait).try_downcast::<dyn TestTrait + Send + Sync>().is_none());
    assert!((&TestStruct as &(dyn BaseTrait + Send)).try_downcast::<dyn TestTrait + Send + Sync>().is_none());
    assert!((&TestStruct as &(dyn BaseTrait + Sync)).try_downcast::<dyn TestTrait + Send + Sync>().is_none());
    assert!((&TestStruct as &(dyn BaseTrait + Send + Sync)).try_downcast::<dyn TestTrait + Send + Sync>().is_some());
}
