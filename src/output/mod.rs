use std::marker::Send;
use std::io::Write;
use std::sync::{Arc};
use std::sync::{RwLock, LockResult, RwLockReadGuard};

pub mod file;

pub trait Output: Sync + Send + 'static {
    fn push(&self, string: &str);
    fn has_color(&self) -> bool;
}

pub trait ReadLock<T>: Sync + Send
    where T: Write
{
    fn arc_lock(&self) -> Arc<RwLock<T>>;
}


pub struct OutputLock<T: Write> {
    lock: Arc<RwLock<T>>,
    color: bool
}

impl<T> OutputLock<T>
    where T: Write + Send + Sync + 'static
{
    #[inline]
    pub fn new(dir: T, color: bool) -> Self {
        OutputLock {
            lock: Arc::new(RwLock::new(dir)),
            color: color
        }
    }
}

impl<T> ReadLock<T> for OutputLock<T>
    where T: Write + Send + Sync + 'static
{
    fn arc_lock(&self) -> Arc<RwLock<T>> {
        self.lock.clone()
    }
}

impl<T> Output for OutputLock<T>
    where T: Write + Send + Sync + 'static
{
    #[inline]
    fn push(&self, string: &str)
    {
        let mut output = self.lock.write().unwrap();
        let _ = writeln!(&mut output, "{}", string);
    }
    fn has_color(&self) -> bool {
        self.color
    }
}