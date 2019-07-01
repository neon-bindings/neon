use crate::types::Value;

pub trait SuperType<T: Value> {
    fn upcast_internal(_: T) -> Self;
}
