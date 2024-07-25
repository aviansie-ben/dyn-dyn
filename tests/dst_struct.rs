use dyn_dyn::{dyn_dyn_base, dyn_dyn_cast, dyn_dyn_impl, DynDynBase, DynDynTable};

#[dyn_dyn_base]
trait Base {}
trait Trait {
    fn test(&self) -> u32;
}

struct StructA;

#[dyn_dyn_impl]
impl Base for StructA {}

struct StructB(u32);

#[dyn_dyn_impl(Trait)]
impl Base for StructB {}
impl Trait for StructB {
    fn test(&self) -> u32 {
        self.0
    }
}

#[allow(dead_code)]
struct DstStruct<T: ?Sized>(u32, T);

unsafe impl<B: ?Sized + DynDynBase> DynDynBase for DstStruct<B> {
    fn get_dyn_dyn_table(&self) -> DynDynTable {
        B::get_dyn_dyn_table(&self.1)
    }
}

#[test]
fn test_dst_struct_cast() {
    assert_eq!(
        Err(()),
        dyn_dyn_cast!(Base => Trait [DstStruct<$>], &DstStruct(0, StructA) as &DstStruct<dyn Base>)
            .map(|t| t.1.test())
            .map_err(|_| ())
    );
    assert_eq!(
        Ok(1234),
        dyn_dyn_cast!(Base => Trait [DstStruct<$>], &DstStruct(0, StructB(1234)) as &DstStruct<dyn Base>)
            .map(|t| t.1.test())
            .map_err(|_| ())
    );
}
