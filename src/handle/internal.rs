use value::Value;

pub trait SuperType<T: Value> {
    fn upcast_internal(T) -> Self;
}
