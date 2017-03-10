use std::marker::Send;

pub mod stdout;

pub trait ReceiverData: Sized + Send + Sync + 'static {
    fn get_direction(&self) -> &super::Direction;
    fn get_string(&self) -> &str;
}

pub trait Output<T: ReceiverData>: Clone + Send + Sync + 'static {
    fn push(&self, input: T);
}

