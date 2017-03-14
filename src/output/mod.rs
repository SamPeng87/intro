use std::marker::Send;

pub mod stdout;
use super::LogEntry;
use std::sync::Arc;

pub trait ReceiverData: Send + Sync + 'static {
    fn get_direction(&self) -> &super::Direction;
    fn get_string(&self) -> &str;
}

pub trait Output: Send + Sync + 'static {
    fn push(&self, input: Arc<LogEntry>);
}

