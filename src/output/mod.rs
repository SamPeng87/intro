use std::marker::Send;

pub mod stdout;

pub trait ReceiverData: Sized + Send + Sync + 'static {
    type Type: ? Sized;
    fn get_direction(&self) -> &Self::Type;
    fn get_string(&self) -> &str;
}

pub trait Output<T: ReceiverData>: Send + Sync + 'static {
    fn push(&self, input: T);
}

