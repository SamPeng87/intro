use output::Output;
use output::OutputLock;
use output::file::File as InnerFile;
use std::sync::{Arc};
use LogEntry;
use Channeled;
use Formatter;
use format::StringFormatter;
use std::mem;
use std::collections::HashMap;
use crossbeam;
use std::sync::mpsc::{Sender, channel, Receiver};
use std::sync::Mutex;
use std::io;
use log::{LogLevel, LogLevelFilter, LogLocation, SetLoggerError, LogMetadata, LogRecord};
use std::thread;
use crossbeam::sync::MsQueue;

struct EventRouter {
    formatter: Arc<Formatter>,
    output: Vec<Arc<Output>>,
}

pub struct EventRouterBuilder {
    formatter: Arc<Formatter>,
    output: Vec<Arc<Output>>,
}


impl EventRouter {
    #[inline]
    fn gateway(&self, queue: &Arc<MsQueue<OutputEntry>>, record: Arc<LogEntry>) {
        for output in &self.output {
            let entry = OutputEntry {
                formatter: self.formatter.clone(),
                output: output.clone(),
                entry: record.clone(),
            };
            queue.push(entry);
        }
    }
}

impl EventRouterBuilder
{
    pub fn new(formater: Arc<Formatter>)
               -> Self
    {
        EventRouterBuilder {
            formatter: formater.clone(),
            output: vec!(),
        }
    }

    pub fn add<T: Output>(&mut self, output: Arc<T>) -> &mut Self {
        self.output.push(output);
        self
    }

    fn build(&mut self) -> EventRouter {
        EventRouter {
            formatter: self.formatter.clone(),
            output: self.output.clone(),
        }
    }
}

pub struct EventRouterFilterBuilder {
    router: HashMap<Option<i32>, Vec<EventRouter>>,
}

impl EventRouterFilterBuilder {
    #[inline]
    pub fn new() -> EventRouterFilterBuilder {
        EventRouterFilterBuilder {
            router: HashMap::new(),
        }
    }

    pub fn add(&mut self, level: LogLevelFilter, router: &mut EventRouterBuilder) -> &mut Self {
        self.router.entry(Some(level.to_log_level().unwrap() as i32)).or_insert(Vec::new()).push(router.build());
        self
    }
    pub fn default(&mut self, router: &mut EventRouterBuilder) -> &mut Self {
        self.router.entry(None).or_insert(Vec::new()).push(router.build());
        self
    }

    #[inline]
    pub fn build(&mut self) -> &mut HashMap<Option<i32>, Vec<EventRouter>> {
        &mut self.router
    }
}

struct OutputEntry {
    entry: Arc<LogEntry>,
    output: Arc<Output>,
    formatter: Arc<Formatter>,
}

pub struct FileChannel {
    queue: Arc<MsQueue<OutputEntry>>,
    router: HashMap<Option<i32>, Vec<EventRouter>>,
}

impl FileChannel {
    pub fn new(builder: &mut EventRouterFilterBuilder) -> FileChannel {
        let queue = Arc::new(MsQueue::new());

        unsafe {
            let q = queue.clone();
            crossbeam::spawn_unsafe(move || FileChannel::work(q));
        }

        FileChannel {
            queue: queue.clone(),
            router: mem::replace(builder.build(), HashMap::new())
        }
    }
    fn work(rx: Arc<MsQueue<OutputEntry>>) {
        loop {
            let entry = rx.clone().pop();
            let data = entry.formatter.parse(&(entry.entry));
            entry.output.push(data.as_str());
            drop(entry)
        }
    }
}

impl Channeled for FileChannel {
    fn send(&self, data: Arc<LogEntry>) {
        let routers = match self.router.get(&Some(data.level() as i32)) {
            Some(router) => {
                router
            }
            None => {
                self.router.get(&None).expect("this channel not have default executer")
            }
        };

        for r in routers {
            r.gateway(&self.queue, data.clone());
        }
    }
}

#[test]
fn test_file_channel() {
//    use std::thread;
//    use std::time::SystemTime;

//    let channel = FileChannel::new();
//    let now = SystemTime::now();
//
//    for i in 0..10000000 {
//        let entry = LogEntry {
//            location: LogLocation {
//                __module_path: "123",
//                __file: "321",
//                __line: 12,
//            },
//            msg: "test".to_string(),
//            level: LogLevel::Warn,
//        };
//        let now = SystemTime::now();
//        let test = Arc::new(entry);
//        channel.send(test.clone());
//    }
//
//    thread::sleep_ms(3000000);
}

