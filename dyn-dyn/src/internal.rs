use crate::{DynDyn, DynDynTable};

pub unsafe trait DynDynDerived<B: ?Sized + DynDyn<B>> {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

pub trait DynDynImpl<B: ?Sized> {
    const IS_SEND: bool;
    const IS_SYNC: bool;

    fn get_dyn_dyn_table(&self) -> DynDynTable;
}
