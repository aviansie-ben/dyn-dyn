use crate::DynDynTable;

pub unsafe trait DynDynBase<BaseMarker: ?Sized> {
    fn get_dyn_dyn_table(&self) -> DynDynTable;
}

pub trait DynDynImpl<B: ?Sized> {
    type BaseMarker: ?Sized;

    const IS_SEND: bool;
    const IS_SYNC: bool;

    fn get_dyn_dyn_table(&self) -> DynDynTable;
}
