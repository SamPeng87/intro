#![allow(dead_code)]
#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate regex;
extern crate time;
extern crate crossbeam;
extern crate chrono;
extern crate ansi_term;

use log::{LogLevel, LogLevelFilter, LogLocation, SetLoggerError, LogMetadata, LogRecord};
use std::collections::HashMap;
use std::mem;


mod format;
mod output;
mod channel;
mod level_color;

use std::sync::{Arc};
use time::{Timespec};

const DEFAULT_FORMAT_STRING: &'static str = "%{datetime:rfc3339}\t%{level}:\t%{modulePath}\t%{message}";

type LogExactExecutors = HashMap<&'static str, HashMap<&'static str, LogExecute>>;
type LogModuleExecutors = HashMap<&'static str, LogExecute>;
type LogTargetExecutors = HashMap<&'static str, LogExecute>;


pub trait Channeled: Send + Sync {
    fn send(&self, strings: Arc<LogEntry>);
}

pub trait Parted {
    fn name(&self) -> &str;
    fn layout(&self) -> &Option<String>;
}


pub trait Formatter: Send + Sync {
    fn parse(&self, color: bool, record: &LogEntry) -> String;
}


#[derive(Copy, Eq, Debug)]
struct LogDirective {
    name: Option<&'static str>,
    level: LogLevelFilter,
}

pub struct LogEntry {
    location: LogLocation,
    msg: String,
    level: LogLevel,
    time: Timespec,
}

impl LogEntry {
    pub fn location(&self) -> &LogLocation {
        &self.location
    }

    pub fn args(&self) -> &str {
        &self.msg
    }

    pub fn level(&self) -> LogLevel {
        self.level
    }
}


impl Clone for LogDirective {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl PartialEq for LogDirective {
    #[inline]
    fn eq(&self, other: &LogDirective) -> bool {
        self.name == other.name && self.level == other.level
    }
}


struct LogExecute {
    //控制过滤条件到指定的channel
    channels: HashMap<Option<i32>, Vec<Arc<Channeled>>>,
}

impl LogExecute {
    #[inline]
    fn control(&self, record: &LogRecord) {
        let level = record.level() as usize();

        let now = time::get_time();

        let entry = LogEntry {
            level: record.level(),
            msg: format!("{}", record.args()),
            location: record.location().clone(),
            time: now,
        };
        let entry_arc = Arc::new(entry);

        for (level_key, channels) in &self.channels {
            match level_key {
                &Some(l) => {
                    if level > l as usize {
                        return
                    }
                }
                &None => {}
            }


            for channel in channels {
                channel.clone().send(entry_arc.clone());
            }
        }

    }
}

struct LogExecuteBuilder {
    channels: HashMap<Option<i32>, Vec<Arc<Channeled>>>,
}

impl LogExecuteBuilder {
    pub fn new() -> LogExecuteBuilder {
        LogExecuteBuilder {
            channels: HashMap::new(),
        }
    }


    pub fn default_channels(&mut self, channeled: Arc<Channeled>) -> &mut Self {
        self.channels.entry(None).or_insert(Vec::new()).push(channeled);
        self
    }

    pub fn add_channels(&mut self, level: LogLevelFilter, channeled: Arc<Channeled>) -> &mut Self {
        self.channels.entry(Some(level.to_log_level().unwrap() as i32)).or_insert(Vec::new()).push(channeled);
        self
    }

    pub fn build(&mut self) -> LogExecute {
        LogExecute {
            channels: mem::replace(&mut self.channels, HashMap::new()),
        }
    }
}

#[allow(dead_code)]
struct Logger {
    default: Option<LogExecute>,
    exact_executors: LogExactExecutors,
    target_executors: LogTargetExecutors,
    module_executors: LogModuleExecutors,
}

impl Logger {
    #[inline]
    fn control(&self, record: &LogRecord) {
        let target = &record.target();
        let location = &record.location();
        let module = location.module_path();


        match self.find_exact(module, target).or_else(|| {
            self.find_target(target)
        }).or_else(|| {
            self.find_module(module)
        }).or_else(|| {
            self.find_default()
        }) {
            Some(execute) => {
                execute.control(record);
            }
            None => {}
        }
    }

    #[inline]
    fn find_exact(&self, module: &str, target: &str) -> Option<&LogExecute> {
        self.exact_executors.get(module).map_or(None, |x| {
            x.get(target)
        })
    }

    #[inline]
    fn find_target(&self, target: &str) -> Option<&LogExecute> {
        self.target_executors.get(target)
    }

    #[inline]
    fn find_module(&self, module: &str) -> Option<&LogExecute> {
        self.module_executors.get(module)
    }

    #[inline]
    fn find_default(&self) -> Option<&LogExecute> {
        self.default.as_ref()
    }
}


impl log::Log for Logger {
    #[allow(unused_variables)]
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &LogRecord) {
        self.control(record);
    }
}

#[allow(dead_code)]
struct LoggerBuilder {
    default: Option<LogExecute>,
    exact_executors: LogExactExecutors,
    target_executors: LogTargetExecutors,
    module_executors: LogModuleExecutors,
    max_level: LogLevelFilter,
}


#[allow(dead_code)]
impl LoggerBuilder {
    fn new() -> Self {
        LoggerBuilder {
            default: None,
            exact_executors: LogExactExecutors::new(),
            target_executors: LogTargetExecutors::new(),
            module_executors: LogModuleExecutors::new(),
            max_level: LogLevelFilter::Trace,
        }
    }

    #[inline]
    fn default(&mut self, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.default = Some(builder.build());
        self
    }

    #[inline]
    fn target(&mut self, target: &'static str, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.target_executors.insert(target, builder.build());
        self
    }

    #[inline]
    fn module(&mut self, module: &'static str, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.module_executors.insert(module, builder.build());
        self
    }

    #[inline]
    fn exact(&mut self, module: &'static str, target: &'static str, builder: &mut LogExecuteBuilder) -> &mut Self {
        self.exact_executors.entry(module).or_insert(HashMap::new()).insert(target, builder.build());
        self
    }

    #[inline]
    fn set_max_logger(&mut self, max: LogLevelFilter) -> &mut Self {
        self.max_level = max;
        self
    }


    fn build(&mut self) -> Logger {
        Logger {
            default: mem::replace(&mut self.default, None),
            exact_executors: mem::replace(&mut self.exact_executors, LogExactExecutors::new()),
            target_executors: mem::replace(&mut self.target_executors, LogTargetExecutors::new()),
            module_executors: mem::replace(&mut self.module_executors, LogModuleExecutors::new()),
        }
    }

    fn init_logger(&mut self) -> Result<(), SetLoggerError> {
        log::set_logger(|max_level| {
            max_level.set(self.max_level);
            Box::new(self.build())
        })
    }
}


#[test]
fn format_parse() {
    use std::thread;
    use std::time::{SystemTime};
    use std::fs::OpenOptions;
    use std::io;
    use output::*;
    use channel::*;

    let file = output::file::File::new("./a/a").expect("can't open file");

    let o1 = Arc::new(OutputLock::new(io::stdout(), true));

    let o2 = Arc::new(OutputLock::new(file, false));

    let formatter1 = Arc::new(format::StringFormatter::new(DEFAULT_FORMAT_STRING));
    //    let formatter2 = Arc::new(format::StringFormatter::new("%{message}"));

    let mut event_router_builder = single_channel::EventRouterBuilder::new(formatter1);
    event_router_builder.add(o1);
    //    event_router_builder.add(o2);

    let mut event_router_filter_builder = single_channel::EventRouterFilterBuilder::new();
//    event_router_filter_builder.add(LogLevelFilter::Debug, &mut event_router_builder);
    event_router_filter_builder.default(&mut event_router_builder);


    let channel = Arc::new(single_channel::FileChannel::new(&mut event_router_filter_builder));

    let mut execute = LogExecuteBuilder::new();

    execute
        .add_channels(LogLevelFilter::Info, channel.clone());

    LoggerBuilder::new()
        .module(module_path!(), &mut execute)
        .set_max_logger(LogLevelFilter::Trace)
        .init_logger();

    let now = SystemTime::now();
    info!("{}", "test o 1111111111 22222222222 3333333333");
    debug!("{}", "test o 1111111111 22222222222 3333333333");
    error!("{}", "test o 1111111111 22222222222 3333333333");
    warn!("{}", "test o 1111111111 22222222222 3333333333");
    trace!("{}", "test o 1111111111 22222222222 3333333333");
    info!("{}", "test o 1111111111 22222222222 3333333333");


    match now.elapsed() {
        Ok(elapsed) => {
            // it prints '2'
            println!("time is {}", elapsed.subsec_nanos());
        }
        Err(e) => {
            // an error occured!
            println!("Error: {:?}", e);
        }
    };
    thread::sleep_ms(2000);


    //            use std::thread;
    //    //            use std::time::{SystemTime};
    //
    //    //    let _ = init();
    //    //    let mut children = vec![];
    //
    ////    for i in 0..5 {
    ////        thread::spawn(move || {
    ////            let now = SystemTime::now();
    ////            for j in 0..1000 {
    ////                error!(target: "test", "{} {} ", i, j );
    ////            }
    ////            match now.elapsed() {
    ////                Ok(elapsed) => {
    ////                    // it prints '2'
    ////                    println!("time is {}", elapsed.subsec_nanos());
    ////                }
    ////                Err(e) => {
    ////                    // an error occured!
    ////                    println!("Error: {:?}", e);
    ////                }
    ////            }
    ////        });
    ////    }
    ////    for child in children {
    ////        child.join().unwrap()
    ////    }
    //    thread::sleep_ms(30000);
}

