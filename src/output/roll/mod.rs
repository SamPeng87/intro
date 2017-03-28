use output::file::File;
use output::ReadLock;
use std::sync::Arc;
use std::thread;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::os::unix::fs::MetadataExt;
use std::mem;
use std::io::Write;

pub mod time;

trait Strategy: Send + Sync {
    fn enable(&self, file: &File) -> bool;
}

struct Roll {
    file: Arc<ReadLock<File>>,
    strategy: Arc<Strategy>
}

struct Inner {
    file: Vec<Roll>
}

struct RollFile {
    inner: Arc<Inner>,
}

fn work(inner: &Inner) {
    loop {
        let _:Vec<_> = inner.file
            .iter()
            .map(|x| {
                thread::sleep_ms(1);
                let lock = x.file.clone().arc_lock();
                let file = lock.read().unwrap();
                if x.strategy.enable(&file) {
                    println!("need roll this file {:?}", &file.metadata().unwrap().size());
                };
            }).collect();
    }
}

pub struct RoleBuilder {
    roles: Vec<Roll>
}

impl RoleBuilder {
    pub fn new() -> RoleBuilder {
        RoleBuilder {
            roles: Vec::new()
        }
    }

    pub fn add<T>(&mut self, f: Arc<ReadLock<File>>, strategy: Arc<T>) -> &mut Self
        where T: Strategy + 'static

    {
        self.roles.push(Roll {
            file: f.clone(),
            strategy: strategy.clone()
        });
        self
    }

    pub fn build(&mut self) -> RollFile {
        let inner = Arc::new(Inner {
            file: mem::replace(&mut self.roles, Vec::new())
        });

        RollFile {
            inner: inner.clone()
        }
    }
}


impl RollFile {
    pub fn run(&mut self) {
        let inner = self.inner.clone();
        thread::Builder::new()
            .name(format!("{}", "file-roll-strategy"))
            .spawn(move || work(&inner)).unwrap();
    }
}

