use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_impl};

#[dyn_dyn_base]
trait BaseTrait {}
trait TestTrait {
    fn test(&self) -> u32;
}

struct TestStruct;

#[dyn_dyn_impl(TestTrait)]
impl BaseTrait for TestStruct {}
impl TestTrait for TestStruct {
    fn test(&self) -> u32 {
        0xdeadbeef
    }
}

#[test]
fn test_plain_dyn_from_send_sync() {
    assert_eq!(
        Ok(0xdeadbeef),
        dyn_dyn_cast!(BaseTrait + Send => TestTrait, &TestStruct as &(dyn BaseTrait + Send))
            .map(|t| t.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(0xdeadbeef),
        dyn_dyn_cast!(BaseTrait + Sync => TestTrait, &TestStruct as &(dyn BaseTrait + Sync))
            .map(|t| t.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(0xdeadbeef),
        dyn_dyn_cast!(BaseTrait + Send + Sync => TestTrait, &TestStruct as &(dyn BaseTrait + Send + Sync))
            .map(|t| t.test()).map_err(|_| ())
    );
}

#[test]
fn test_send_dyn() {
    assert_eq!(
        Ok(0xdeadbeef),
        dyn_dyn_cast!(BaseTrait + Send => TestTrait + Send, &TestStruct as &(dyn BaseTrait + Send))
            .map(|t| t.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(0xdeadbeef),
        dyn_dyn_cast!(BaseTrait + Send + Sync => TestTrait + Send, &TestStruct as &(dyn BaseTrait + Send + Sync))
            .map(|t| t.test()).map_err(|_| ())
    );
}

#[test]
fn test_sync_dyn() {
    assert_eq!(
        Ok(0xdeadbeef),
        dyn_dyn_cast!(BaseTrait + Sync => TestTrait + Sync, &TestStruct as &(dyn BaseTrait + Sync))
            .map(|t| t.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(0xdeadbeef),
        dyn_dyn_cast!(BaseTrait + Send + Sync => TestTrait + Sync, &TestStruct as &(dyn BaseTrait + Send + Sync))
            .map(|t| t.test()).map_err(|_| ())
    );
}

#[test]
fn test_send_sync_dyn() {
    assert_eq!(
        Ok(0xdeadbeef),
        dyn_dyn_cast!(
            BaseTrait + Send + Sync => TestTrait + Send + Sync,
            &TestStruct as &(dyn BaseTrait + Send + Sync)
        )
        .map(|t| t.test())
        .map_err(|_| ())
    );
}
