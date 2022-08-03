use crate::{DynDyn, DynDynTable};

pub unsafe trait DynDynDerived<B: ?Sized + DynDyn> {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

pub trait DynDynImpl {
    type BaseDynDyn: ?Sized + DynDynImpl;

    const IS_SEND: bool;
    const IS_SYNC: bool;

    fn get_dyn_dyn_table(&self) -> DynDynTable;
}
