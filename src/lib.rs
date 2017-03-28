#![allow(dead_code)]
#[macro_use]
extern crate log;

extern crate regex;
extern crate time;
extern crate rand;
extern crate futures;
extern crate futures_cpupool;

use log::{LogLevel, LogLevelFilter, LogLocation, SetLoggerError, LogMetadata, LogRecord};
use std::collections::HashMap;
use std::mem;


mod format;
mod output;
mod channel;

use std::sync::{Arc};

const DEFAULT_FORMAT_STRING: &'static str = "%{level}:\t%{modulePath}\t%{message}";

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
    fn parse(&self, record: &LogEntry) -> String;
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
    outputs: HashMap<Option<i32>, Vec<Arc<Channeled>>>,
    //控制是否输出
    directive: Vec<LogDirective>,
}

impl LogExecute {
    #[inline]
    fn control(&self, record: &LogRecord) {
        if self.decision_directive(record) {
            let outputs = self.decision_outouts(record);

            let entry = LogEntry {
                level: record.level(),
                msg: format!("{}", record.args()),
                location: record.location().clone(),
            };
            let entry_arc = Arc::new(entry);

            for output in outputs {
                output.clone().send(entry_arc.clone());
            }
        }
    }
    #[inline]
    fn decision_outouts(&self, record: &LogRecord) -> &Vec<Arc<Channeled>> {
        let level = Some(record.level() as i32);
        match self.outputs.get(&level) {
            Some(output) => {
                output
            },
            None => {
                self.outputs.get(&None).expect("no have any default outputs for this log execute")
            }
        }
    }

    #[inline]
    fn decision_directive(&self, record: &LogRecord) -> bool {
        let level = record.level();
        for dir in self.directive.iter().rev() {
            match dir.name {
                Some(..) | None => {
                    return level <= dir.level
                }
            }
        }
        false
    }
}

struct LogExecuteBuilder {
    outputs: HashMap<Option<i32>, Vec<Arc<Channeled>>>,
    directive: Vec<LogDirective>,
}

impl LogExecuteBuilder {
    pub fn new() -> LogExecuteBuilder {
        LogExecuteBuilder {
            outputs: HashMap::new(),
            directive: vec!(),
        }
    }

    pub fn add_log_directive(&mut self, name: Option<&'static str>, level: LogLevelFilter) -> &mut Self {
        self.directive.push(LogDirective {
            name: name,
            level: level,
        });
        self
    }

    pub fn default_output(&mut self, channeled: Arc<Channeled>) -> &mut Self {
        self.outputs.entry(None).or_insert(Vec::new()).push(channeled);
        self
    }

    pub fn add_output(&mut self, level: LogLevelFilter, channeled: Arc<Channeled>) -> &mut Self {
        self.outputs.entry(Some(level.to_log_level().unwrap() as i32)).or_insert(Vec::new()).push(channeled);
        self
    }

    pub fn build(&mut self) -> LogExecute {
        LogExecute {
            directive: mem::replace(&mut self.directive, vec!()),
            outputs: mem::replace(&mut self.outputs, HashMap::new()),
        }
    }
}

//type LogExecutors = HashMap<&'static str, HashMap<&'static str, LogExecute>>;
//type LogModuleExecutors = HashMap<&'static str, LogExecute>;
//type LogTargetExecutors = HashMap<&'static str, LogExecute>;

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
            },
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


#[cfg(test)]
mod tests;

#[test]
fn format_parse() {
    use std::thread;
    use std::time::{SystemTime};
    use std::fs::OpenOptions;
    use std::io;
    use output::*;
    use channel;
    use channel::EventRouterBuilder;
    use std::os::unix::io::AsRawFd;



    let file = output::file::File::new("./a/a").expect("can't open file");

    let fd = file.as_raw_fd();

    let o1 = Arc::new(OutputLock::new(io::stdout()));
    let o2 = Arc::new(output::OutputLock::new(file));




    let timer_strategy = roll::time::TimeStrategy {};

    let mut roll = roll::RoleBuilder::new().add(o2.clone(),Arc::new(timer_strategy)).build();
    roll.run();




    let formatter1 = Arc::new(format::StringFormatter::new(DEFAULT_FORMAT_STRING));
    let formatter2 = Arc::new(format::StringFormatter::new("%{message}"));


    let mut event_router_builder = EventRouterBuilder::new(formatter1);

    event_router_builder.add(o1.clone());

    let mut event_router_builder2 = EventRouterBuilder::new(formatter2);

    event_router_builder2.add(o2.clone());

    let mut event_pool_builder = channel::EventPoolBuilder::new(1);

    event_pool_builder.add(LogLevelFilter::Info, &mut event_router_builder);
    event_pool_builder.add(LogLevelFilter::Error, &mut event_router_builder2);
//    event_pool_builder.add(LogLevelFilter::Error, &mut event_router_builder);
    event_pool_builder.default(&mut event_router_builder);


    let channel = Arc::new(event_pool_builder.build());


    let mut execute = LogExecuteBuilder::new();

    execute
        .add_log_directive(None, LogLevelFilter::Info)
        .default_output(channel.clone())
        .add_output(LogLevelFilter::Info, channel.clone());

    //    //    let mut execute2 = LogExecuteBuilder::new();
    //    //
    //    //    execute2
    //    //        .add_log_directive(None, LogLevelFilter::Info)
    //    //        .add_output(LogLevelFilter::Info, Box::new(StdChannel::new(event::EventPool::new(o2.clone()), o2.clone())))
    //    //        .set_sync(false);
    //
    LoggerBuilder::new()
        .module(module_path!(), &mut execute)
        .set_max_logger(LogLevelFilter::Info)
        .init_logger();
    //
    let now = SystemTime::now();
    error!("{}", "test o 1111111111 22222222222 3333333333");
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
    //    thread::sleep_ms(2000);


    //            use std::thread;
    //            use std::time::{SystemTime};

    //    let _ = init();
    //    let mut children = vec![];

    for i in 0..5 {
        thread::spawn(move || {
            let now = SystemTime::now();
            for j in 0..1000 {
                error!(target: "test", "{} {} ", i, j );
            }
            match now.elapsed() {
                Ok(elapsed) => {
                    // it prints '2'
                    println!("time is {}", elapsed.subsec_nanos());
                }
                Err(e) => {
                    // an error occured!
                    println!("Error: {:?}", e);
                }
            }
        });
    }
//    for child in children {
//        child.join().unwrap()
//    }
    thread::sleep_ms(30000);
}
