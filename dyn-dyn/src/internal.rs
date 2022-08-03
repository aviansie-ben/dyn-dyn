use crate::DynDynTable;

pub unsafe trait DynDynDerived<B: ?Sized> {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

pub trait DynDynImpl<B: ?Sized> {
    const IS_SEND: bool;
    const IS_SYNC: bool;

    fn get_dyn_dyn_table(&self) -> DynDynTable;
}
